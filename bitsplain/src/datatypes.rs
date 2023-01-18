use crate::basic::bytes;
use crate::value::*;
use crate::*;
use bitcoin::blockdata::constants::genesis_block;
use bitcoin::hashes::Hash;
use bitcoin::{BlockHash, Network};
use rust_decimal::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct Sat(Decimal);

const SATS: Decimal = Decimal::from_parts(100000000, 0, 0, false, 0);

impl Sat {
    pub fn new(sat: u64) -> Sat {
        Sat(Decimal::from_u64(sat).unwrap())
    }

    pub fn sat(&self) -> u64 {
        self.0.to_u64().unwrap()
    }

    pub fn btc(&self) -> Decimal {
        self.0 / SATS
    }

    pub fn as_str(&self) -> String {
        format!("{} ₿", self.btc())
    }
}

/// Internal representation of chain hash.
#[derive(Clone, Debug)]
pub struct ChainHash {
    pub block_hash: BlockHash,
    pub network: Option<Network>,
}

impl ChainHash {
    pub fn as_string(&self) -> String {
        match self.network {
            Some(n) => n.to_string(),
            None => "unknown".to_string(),
        }
    }
}

impl ToValue for ChainHash {
    fn to_value(&self) -> Value {
        Value::Alt(
            Box::new(Value::Hash(self.block_hash.as_hash())),
            Box::new(Value::String(self.as_string())),
        )
    }
}

/// Parser of chain hash.
pub fn chain_hash_le(s: Span) -> IResult<Span, ChainHash> {
    let (s, mut b) = bytes(32_usize)(s)?;

    b.reverse();

    let block_hash = BlockHash::from_slice(&b).unwrap();

    let network = if block_hash == genesis_block(Network::Bitcoin).block_hash() {
        Some(Network::Bitcoin)
    } else if block_hash == genesis_block(Network::Regtest).block_hash() {
        Some(Network::Regtest)
    } else if block_hash == genesis_block(Network::Testnet).block_hash() {
        Some(Network::Testnet)
    } else if block_hash == genesis_block(Network::Signet).block_hash() {
        Some(Network::Signet)
    } else {
        None
    };

    Ok((
        s.with("datatype", "chain_hash"),
        ChainHash {
            block_hash,
            network,
        },
    ))
}

/// Parser of chain hash.
pub fn chain_hash(s: Span) -> IResult<Span, ChainHash> {
    let (s, b) = bytes(32_usize)(s)?;

    let block_hash = BlockHash::from_slice(&b).unwrap();

    let network = if block_hash == genesis_block(Network::Bitcoin).block_hash() {
        Some(Network::Bitcoin)
    } else if block_hash == genesis_block(Network::Regtest).block_hash() {
        Some(Network::Regtest)
    } else if block_hash == genesis_block(Network::Testnet).block_hash() {
        Some(Network::Testnet)
    } else if block_hash == genesis_block(Network::Signet).block_hash() {
        Some(Network::Signet)
    } else {
        None
    };

    Ok((
        s.with("datatype", "chain_hash"),
        ChainHash {
            block_hash,
            network,
        },
    ))
}
