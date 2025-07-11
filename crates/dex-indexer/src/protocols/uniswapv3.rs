use std::{str::FromStr, sync::Arc};

use ethers::{
    abi::RawLog, contract::{abigen, EthEvent}, core::types::{Address, Log, H256, U256}, providers::{Http, Provider}
};
use eyre::{ensure, Ok, Result};
use serde::Deserialize;

use crate::types::{UNISWAP_V2_SWAP_TOPIC, UNISWAP_V2_SWAP_TOPIC};

abigen!(
    IUniswapV3Pool,
    r#"[
        event Swap(address indexed sender, address indexed recipient, int256 amount0, int256 amount1, uint160 sqrtPriceX96, uint128 liquidity, int24 tick)
        function token0() external view returns (address)
        function token1() external view returns (address)
    ]"#,
);

abigen!(
    IUniswapV3Factory,
    r#"[
        event PoolCreated(address indexed token0, address indexed token1, uint24 indexed fee, int24 tickSpacing, address pool)
    ]"#,
);

const V2FACTORY_ADDRESS: &str = "0x1F98431c8aD98523631AE4a59f267346ea31F984";

pub fn uniswapv3_event_filter(block: u64) -> Filter {
    Filter::new()
        .address(V2FACTORY_ADDRESS.parse::<Address>()?)
        .event("PoolCreated(address,address,uint24,int24,address)")
        .from_block(block)
}

#[derive(Debug, Clone, Deserialize)]
pub struct UniswapV3PoolCreated {
    pub pool: Address,
    pub token0: Address,
    pub token1: Address,
    pub fee: u32,
}

impl TryFrom<&Log> for UniswapV3PoolCreated {
    type Error = eyre::Error;

    fn try_from(value: &Log) -> std::result::Result<Self> {
        let token0 = Address::from(log.topics[1]);
        let token1 = Address::from(log.topics[2]);
        let fee = U256::from_big_endian(&log.topics[3].as_bytes()[29..32]);
        let pool = Address::from(&log.data[44..64].try_into()?);

        Ok(Self { 
            pool,
            token0,
            token1,
            fee
        })
    }
}

impl UniswapV3PoolCreated {
    pub async fn to_pool(&self, provider: Arc<Provider<Http>>) -> Result<Pool> {
        let token0_decimals = get_coin_decimals(provider, &self.token0).await?;
        let token1_decimals = get_coin_decimals(provider, &self.token1).await?;

        let tokens = vec![
            Token::new(&self.token0, token0_decimals),
            Token::new(&self.token1, token1_decimals),
        ];

        let extra = PoolExtra::UniSwapV2 { fee: self.fee };

        Ok(Pool { 
            protocol: Protocol::UniSwapV2,
            pool: self.pair,
            tokens,
            extra 
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct UniswapV3SwapEvent {
    pub pool: Address,
    pub token0: Address,
    pub token1: Address,
    pub amount0: U256,
    pub amount1: U256,
    pub liquidity: u128,
}

impl UniswapV3SwapEvent {
    pub async fn try_from_log(log: &Log, provider: Arc<Provider<Http>>) -> Result<Self> {
        ensure!(
            !log.topics.is_empty() && 
            log.topics[0] == UNISWAP_V3_SWAP_TOPIC,
            "Not a UniswapV3 Swap event"
        );

        let parsed_log: SwapFilter = SwapFilter::decode_log(&RawLog {
            topics: log.topics.clone(),
            data: log.data.to_vec(),
        })?;

        let pool_address = log.address;
        let pool_contract = IUniswapV3Pool::new(pool_address, provider);

        let token0_address: Address = pool_contract.token_0().call().await?;
        let token1_address: Address = pool_contract.token_1().call().await?;

        let (token_0, token_1, amount_0, amount_1, liquility);

        if parsed_log.amount_0 > 0.into() {
            token_0 = token0_address;
            amount_0 = parsed_log.amount_0.into_raw();
            token_1 = token1_address;
            amount_1 = (-parsed_log.amount_1).into_raw();
        } else {
            token_1 = token1_address;
            amount_1 = parsed_log.amount_1.into_raw();
            token_0 = token0_address;
            amount_0 = (-parsed_log.amount_0).into_raw();
        }
        liquility = parsed_log.liquidity;

        Ok(Self {
            pool: pool_address,
            token0: token_0,
            token1: token_1,
            amount0: amount_0,
            amount1: amount_1,
            liquidity: liquility,
        })
    }
}
