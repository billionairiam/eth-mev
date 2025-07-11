pub mod uniswapv2;
pub mod uniswapv3;

use cached::proc_macro::cached;
use ethers::{abi::Address, providers::{Http, Provider}};
use eyre::{bail, ensure, eyre, Ok, Result};

abigen!(
        Erc20,
        r#"[
            function decimals() public pure returns (uint8)
        ]"#,
    );

pub static WETH_ADDRESS: Lazy<Address> = Lazy::new(|| {
    "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".parse().expect("Invalid WETH address")
});

#[cached(key = "Address", convert = r##"{ *coin_address.to_string() }"##, result = true)]
pub async fn get_coin_decimals<M: Middleware>(
    provider: Arc<M>,
    coin_address: &Address,
) -> Result<u8> {
    if coin_address.is_zero() || *coin_address == *WETH_ADDRESS {
        return Ok(18);
    }

    let contract = Erc20::new(*coin_address, provider);
    match contract.decimals().call().await {
        Ok(decimals) => Ok(decimals),
        Err(e) => {
            Err(eyre!(
                "Failed to fetch decimals for token {}: {}",
                coin_address,
                e
            ))
        }
    }
}
