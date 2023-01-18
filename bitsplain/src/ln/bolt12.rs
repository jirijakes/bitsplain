use crate::ann::ann;
use crate::ann::auto;
use crate::bitcoin::PublicKey;
use crate::nom::combinator::success;
use crate::nom::multi::{length_value, many0};
use crate::nom::number::complete::*;
use crate::nom::IResult;
use crate::parse::*;
use crate::types::*;
use crate::value::{ToValue, Value};
use crate::Span;

#[derive(Debug, Clone)]
pub enum Offer {
    ChainHash(ChainHash),
    Description(String),
    Issuer(String),
    Currency(String),
    Other(Vec<u8>),
    PublicKey(PublicKey),
}

impl ToValue for Offer {
    fn to_value(&self) -> Value {
        match self {
            Offer::ChainHash(s) => s.to_value(),
            Offer::Description(s) => Value::text(s),
            Offer::Issuer(s) => Value::text(s),
            Offer::Currency(s) => Value::text(s),
            Offer::Other(b) => Value::bytes(b.to_vec()),
            Offer::PublicKey(pk) => pk.to_value(),
        }
    }
}

pub fn description(s: Span) -> IResult<Span, Offer> {
    let (s, bytes) = many0(u8)(s)?;
    Ok((
        s,
        Offer::Description(String::from_utf8_lossy(&bytes).to_string()),
    ))
}

pub fn issuer(s: Span) -> IResult<Span, Offer> {
    let (s, bytes) = many0(u8)(s)?;
    Ok((
        s,
        Offer::Issuer(String::from_utf8_lossy(&bytes).to_string()),
    ))
}

pub fn currency(s: Span) -> IResult<Span, Offer> {
    let (s, bytes) = many0(u8)(s)?;
    Ok((
        s,
        Offer::Currency(String::from_utf8_lossy(&bytes).to_string()),
    ))
}

// TODO: It should be possible to write a generic function so we don't
// have to write a new one for each case.
//
// as_offer(chain_hash, Offer::ChainHash)
pub fn offer_chain_hash(s: Span) -> IResult<Span, Offer> {
    let (s, ch) = chain_hash(s)?;
    Ok((s, Offer::ChainHash(ch)))
}

pub fn offer_node_id(s: Span) -> IResult<Span, Offer> {
    let (s, pk) = public_key(s)?;
    Ok((s, Offer::PublicKey(pk)))
}

pub fn other(s: Span) -> IResult<Span, Offer> {
    let (s, bytes) = many0(u8)(s)?;
    Ok((s, Offer::Other(bytes)))
}

pub fn tlv_record(s: Span) -> IResult<Span, Offer> {
    let (s, typ) = p(u8, ann("Type", auto()))(s)?;
    let (s, length) = p(u8, ann("Length", auto()))(s)?;
    let (s, value) = p(
        length_value(
            success(length),
            match typ {
                2 => offer_chain_hash,
                6 => currency,
                10 => description,
                18 => issuer,
                22 => offer_node_id,
                _ => other,
            },
        ),
        ann("Value", auto()),
    )(s)?;

    let annotation = match typ {
        2 => "Offer chain",
        4 => "Offer metadata",
        6 => "Offer currency",
        8 => "Offer amount",
        10 => "Offer description",
        18 => "Offer issuer",
        20 => "Offer quantity max",
        22 => "Offer node ID",
        240 => "Signature",
        _ => "Unknown type",
    };

    Ok((s.with("annotation", annotation), value))
}

pub fn bolt12(s: Span) -> IResult<Span, String> {
    let (s, records) = p(
        many0(p(tlv_record, ann("TLV Record", Value::Nil))),
        ann("TLV Stream", Value::Nil),
    )(s)?;

    Ok((s, format!("{records:?}")))
}
