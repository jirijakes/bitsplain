use crate::parse::with;
use crate::value::*;
use crate::*;

use bitcoin::blockdata::constants::genesis_block;
use bitcoin::hashes::{sha256d, Hash};
use bitcoin::secp256k1::ecdsa::Signature;
use bitcoin::{BlockHash, Network};
use bitcoin::{PublicKey, Txid};
use nom::combinator::success;
use nom::multi::length_count;
use nom::number::streaming::*;
use nom::{IResult, Parser, ToUsize};
use rust_decimal::prelude::*;
use time::OffsetDateTime;

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
            Box::new(Value::text(self.as_string())),
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

pub fn bytes<'a, U: ToUsize + std::fmt::Debug + Copy>(
    len: U,
) -> impl Fn(Span<'a>) -> IResult<Span<'a>, Vec<u8>> {
    move |input: Span<'a>| with("datatype", "bytes", length_count(success(len), u8))(input)
}

pub fn sha256d(input: Span) -> IResult<Span, sha256d::Hash> {
    let (s, x) = bytes(32_usize)(input)?;
    let x = sha256d::Hash::from_slice(&x).unwrap();
    Ok((s.with("datatype", "sha256"), x))
}

pub fn txid(input: Span) -> IResult<Span, Txid> {
    let (s, x) = sha256d(input)?;
    Ok((s.with("datatype", "txid"), Txid::from_hash(x)))
}

pub fn signature(input: Span) -> IResult<Span, Signature> {
    let (s, b) = bytes(64_usize)(input)?;
    match Signature::from_compact(&b) {
        Ok(sig) => Ok((s.with("datatype", "signature"), sig)),
        Err(_) => Err(nom::Err::Failure(nom::error::Error {
            input: s,
            code: nom::error::ErrorKind::Fail,
        })),
    }
}

pub fn public_key(input: Span) -> IResult<Span, PublicKey> {
    let (s, b) = bytes(33_usize)(input)?;
    match PublicKey::from_slice(&b) {
        Ok(pk) => Ok((s.with("datatype", "public_key"), pk)),
        Err(_) => Err(nom::Err::Failure(nom::error::Error {
            input: s,
            code: nom::error::ErrorKind::Fail,
        })),
    }
}

fn varint_impl(input: Span) -> IResult<Span, u64> {
    let (s, byte) = le_u8(input)?;

    let s_int = match byte {
        0xfd => {
            let (s, a) = le_u16(s)?;
            (s, a as u64)
        }
        0xfe => {
            let (s, a) = le_u32(s)?;
            (s, a as u64)
        }
        0xff => le_u64(s)?,
        n => success(n as u64)(s)?,
    };
    Ok(s_int)
}

pub fn uint32(input: Span) -> IResult<Span, u32> {
    with("datatype", "uint32", le_u32)(input)
    // p(le_u32, datatype("uint32"))(input)
}

pub fn int32(input: Span) -> IResult<Span, i32> {
    with("datatype", "int32", le_i32)(input)
}

pub fn sat(input: Span) -> IResult<Span, Sat> {
    with("datatype", "sat", le_u64)(input).map(|(s, n)| (s, Sat::new(n)))
}

pub fn varint(input: Span) -> IResult<Span, u64> {
    with("datatype", "varint", varint_impl)(input)
}

/// Unix timestamp parser. Provided parser is used for the numeric value,
/// typically `uint32` or `be_u32`.
pub fn timestamp<'a, Parse>(
    mut parser: Parse,
) -> impl FnMut(Span<'a>) -> IResult<Span, OffsetDateTime>
where
    Parse: Parser<Span<'a>, u32, nom::error::Error<Span<'a>>>,
{
    move |input: Span| {
        parser.parse(input).map(|(s, ts)| {
            (
                s.with("datatype", "timestamp"),
                OffsetDateTime::from_unix_timestamp(ts.into()).unwrap(),
            )
        })
    }
}
