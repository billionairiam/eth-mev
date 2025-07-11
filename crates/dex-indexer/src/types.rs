use std::{
    collections::HashSet, fmt, hash::{Hash, Hasher}, ops::Add, sync::Arc
};
use burberry::{async_trait, Executor};
use ethers::{providers::Provider, core::types::{Address, Log, H256, Filter}};
use eyre::{bail, ensure, Ok};
use serde::{Deserialize, Serialize};

use crate::protocols::uniswapv3::uniswapv3_event_filter;

pub const UNISWAP_V2_SWAP_TOPIC: H256 = H256([
    0xd7, 0x8a, 0xd9, 0x5f, 0xa4, 0x6c, 0x99, 0x4b, 0x65, 0x51, 0xd0, 0xda, 0x85, 0xfc, 0x27, 0x5f, 
    0xe6, 0x13, 0xce, 0x37, 0x65, 0x7f, 0xb8, 0xd5, 0xe3, 0xd1, 0x30, 0x84, 0x01, 0x59, 0xd8, 0x22,
]);

pub const UNISWAP_V3_SWAP_TOPIC: H256 = H256([
    0xc4, 0x20, 0x79, 0xf9, 0x4a, 0x63, 0x50, 0xd7, 0xe6, 0x23, 0x5f, 0x29, 0x17, 0x49, 0x24, 0xf9,
    0x28, 0xcc, 0x2a, 0xc8, 0x18, 0xeb, 0x64, 0xfe, 0xd8, 0x00, 0x4e, 0x11, 0x5f, 0xbc, 0xca, 0x67,
]);



#[derive(Debug, Clone)]
pub struct Pool {
    pub protocol: Protocol,
    pub pool: Address,
    pub tokens: Vec<Token>,
    pub extra: PoolExtra,
}

impl PartialEq for Pool {
    fn eq(&self, other: &Self) -> bool {
        self.pool == other.pool
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub token_address: Address,
    pub decimals: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PoolExtra {
    None,
    UniSwapV2 {
        fee: u64,
    },
    UniSwapV3 {
        fee: u64,
    }
}

impl fmt::Display for Pool {
    // protocol|pool|tokens|extra
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}|{}|{}|{}",
            self.protocol,
            self.pool,
            serde_json::to_string(&self.tokens).unwrap(),
            serde_json::to_string(&self.extra).unwrap(),
        )
    }
}

impl TryFrom<&str> for Pool {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self> {
        let parts: Vec<&str> = value.split("|").collect();
        ensure!(parts.len() == 4, "Invalid pool format: {}", value);

        let protocol = Protocol::try_from(parts[0])?;
        let pool = parts[1].parse::<Address>()?;
        let tokens: Vec<Token> = serde_json::from_str(parts[2])?;
        let extra: PoolExtra = serde_json::from_str(parts[3])?;

        Ok(Pool {
            protocol,
            pool,
            tokens,
            extra, 
        })
    }
}

impl Pool {
    pub fn token0_type(&self) -> Address {
        self.tokens[0].token_address.clone()
    }

    pub fn token1_type(&self) -> Address {
        self.tokens[1].token_address.clone()
    }

    pub fn token_count(&self) -> usize {
        self.tokens.len()
    }

    pub fn token_index(&self, token_address: &Address) -> Option<usize> {
        self.tokens.iter().position(|token| token.token_address == token_address)
    }

    pub fn token(&self, index: usize) -> Option<Token> {
        self.tokens.get(index)
    }

    // (token0_address, token1_address)
    pub fn token01_pair(&self) -> Vec<(Address, Address)> {
        let mut pairs = Vec::new();
        for i in 0..self.tokens.len() {
            for j in i+1..self.tokens.len() {
                pairs.push((self.tokens[i].token_address, self.tokens[j].token_address));
            }
        }

        pairs
    }
}

impl Token {
    pub fn new(token_address: &Address, decimals: u8) -> Self {
        Self {
            token_address: token_address.clone(),
            decimals: decimals
        }
    }
}

#[derive(Debug, Clone)]
pub struct SwapEvent {
    pub protocol: Protocol,
    pub pool: Option<Address>,
    pub coin_in: Vec<Address>,
    pub coin_out: Vec<Address>,
    pub amounts_in: Vec<u64>,
    pub amounts_out: Vec<u64>,
}

impl SwapEvent {
    pub fn pool_address(&self) -> Option<Address> {
        self.pool
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Protocol {
    UniSwapV2,
    UniSwapV3,
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Protocol::UniSwapV2 => write!(f, "uniswapv2"),
            Protocol::UniSwapV3 => write!(f, "uniswapv3"),
        }
    }
}

impl TryFrom<&str> for Protocol {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "uniswapv2" => Ok(Protocol::UniSwapV2),
            "uniswapv3" => Ok(Protocol::UniSwapV3),
            _ => bail!("Unsupported protocol: {}", value),
        }
    }
}

impl TryFrom<&Log> for Protocol {
    type Error = eyre::Error;

    fn try_from(value: &Log) -> Result<Self> {
        
    }
}

impl Protocol {
    pub fn try_from_event_topic(topic: &H256) -> Result<Self> {
        match topic {
            UNISWAP_V2_SWAP_TOPIC => {
                Ok(Protocol::UniSwapV2)
            }
            UNISWAP_V2_SWAP_TOPIC => {
                Ok(Protocol::UniSwapV3)
            }
            _ => bail!("Not interesting")
        }
    }

    pub fn event_filter(&self, block: u64) -> Filter {
        match self {
            Protocol::UniSwapV2 => {
                uniswapv2_event_filter(block)
            }
            Protocol::UniSwapV3 => {
                uniswapv3_event_filter(block)
            }
            _ => todo!(),
        }
    }

    pub async fn eth_event_to_pool(&self, log: &Log, provider: &Provider<Provider>) -> Result<Pool> {
        
    }
}
