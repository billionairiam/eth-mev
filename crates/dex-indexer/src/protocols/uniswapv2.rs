use std::sync::Arc;

use ethers::{
    abi::RawLog, contract::{abigen, EthEvent}, core::{k256::elliptic_curve::consts::U24, types::{Address, Filter, Log, H256, U256}}, providers::{Http, Provider}
};
use eyre::{ensure, Ok, Result};
use serde::Deserialize;

use crate::{protocols::get_coin_decimals, types::{Pool, PoolExtra, Protocol, Token}};

abigen!(
    IUniswapV2Pool,
    r#"[
        event Swap(address indexed sender, uint amount0In, uint amount1In, uint amount0Out, uint amount1Out, address indexed to)
        function token0() external view returns (address)
        function token1() external view returns (address)
    ]"#,
);

abigen!(
    IUniswapV2Factory,
    r#"[
        event PairCreated(address indexed token0, address indexed token1, address pair, uint)
    ]"#,
);

const V2FACTORY_ADDRESS: &str = "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f";

pub fn uniswapv2_event_filter(block: u64) -> Filter {
    Filter::new()
        .address(V2FACTORY_ADDRESS.parse::<Address>()?)
        .event("PairCreated(address indexed token0, address indexed token1, address pair, uint)")
        .from_block(block)
}

#[derive(Debug, Clone, Deserialize)]
pub struct UniswapV2PairCreated {
    pub pair: Address,
    pub token0: Address,
    pub token1: Address,
}

impl TryFrom<&Log> for UniswapV2PairCreated {
    type Error = eyre::Error;

    fn try_from(value: &Log) -> std::result::Result<Self> {
        let token0 = Address::from(log.topics[1]);
        let token1 = Address::from(log.topics[2]);
        let pair = Address::from(&log.data[12..32].try_into()?);

        Ok(Self { 
            pair: pair,
            token0: token0,
            token1: token1, 
        })
    }
}

impl UniswapV2PairCreated {
    pub async fn to_pool(&self, provider: Arc<Provider<Http>>) -> Result<Pool> {
        let token0_decimals = get_coin_decimals(provider, &self.token0).await?;
        let token1_decimals = get_coin_decimals(provider, &self.token1).await?;

        let tokens = vec![
            Token::new(&self.token0, token0_decimals),
            Token::new(&self.token1, token1_decimals),
        ];

        let extra = PoolExtra::UniSwapV2 { fee: 5 };

        Ok(Pool { 
            protocol: Protocol::UniSwapV2,
            pool: self.pair,
            tokens,
            extra 
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct UniswapV2SwapEvent {
    pub pool: Address,
    pub token0: Address,
    pub token1: Address,
    pub amount0_in: U256,
    pub amount1_in: U256,
    pub amount0_out: U256,
    pub amount1_out: U256,
}

impl UniswapV2SwapEvent {
    pub async fn try_from_log(log: &Log, provider: Arc<Provider<Http>>) -> Result<Self> {
        ensure!(
            !log.topics.is_empty() && 
            log.topics[0] == UNISWAP_V2_SWAP_TOPIC,
            "Not a UniswapV3 Swap event"
        );

        let parsed_log: SwapFilter = SwapFilter::decode_log(&RawLog {
            topics: log.topics.clone(),
            data: log.data.to_vec(),
        })?;

        let pool_address = log.address;
        let pool_contract = IUniswapV2Pool::new(pool_address, provider);

        let token0_address: Address = pool_contract.token_0().call().await?;
        let token1_address: Address = pool_contract.token_1().call().await?;

        Ok(Self {
            pool: pool_address,
            token0: token0_address,
            token1: token1_address,
            amount0_in: parsed_log.amount_0_in.into(),
            amount1_in: parsed_log.amount_1_in.into(),
            amount0_out: parsed_log.amount_0_out.into(),
            amount1_out: parsed_log.amount_1_out.into(),
        })
    }
}


