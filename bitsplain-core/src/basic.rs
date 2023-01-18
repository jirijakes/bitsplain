use bitcoin::hashes::{sha256d, Hash};
use bitcoin::secp256k1::ecdsa::Signature;
use bitcoin::{PublicKey, Txid};

use nom::combinator::success;
use nom::multi::length_count;
use nom::number::streaming::*;
use nom::{IResult, Parser, ToUsize};
use time::OffsetDateTime;

use crate::datatypes::Sat;
use crate::parse::with;
use crate::*;

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
