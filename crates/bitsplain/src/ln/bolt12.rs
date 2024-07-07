use bytes::Bytes;

use crate::bitcoin::PublicKey;
use crate::dsl::{ann, auto};
use crate::nom::combinator::{peek, verify};
use crate::nom::multi::{length_count, many0};
use crate::nom::number::complete::*;
use crate::parse::*;
use crate::types::*;
use crate::value::{ToValue, Value};

use super::{short_channel_id, ShortChannelId};

#[derive(Debug, Clone)]
pub enum Offer {
    ChainHashes(Vec<ChainHash>),
    Description(String),
    Issuer(String),
    Currency(String),
    Paths,
    Other(Bytes),
    PublicKey(PublicKey),
}

impl ToValue for Offer {
    fn to_value(&self) -> Value {
        match self {
            Offer::ChainHashes(s) => Value::text(format!("{:?}", s)),
            Offer::Description(s) => Value::text(s),
            Offer::Issuer(s) => Value::text(s),
            Offer::Currency(s) => Value::text(s),
            Offer::Paths => Value::Nil,
            Offer::Other(b) => Value::bytes(b.to_vec()),
            Offer::PublicKey(pk) => pk.to_value(),
        }
    }
}

pub fn description(s: Span) -> Parsed<Offer> {
    let (s, bytes) = many0(u8)(s)?;
    Ok((
        s,
        Offer::Description(String::from_utf8_lossy(&bytes).to_string()),
    ))
}

pub fn issuer(s: Span) -> Parsed<Offer> {
    let (s, bytes) = many0(u8)(s)?;
    Ok((
        s,
        Offer::Issuer(String::from_utf8_lossy(&bytes).to_string()),
    ))
}

pub fn currency(s: Span) -> Parsed<Offer> {
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
pub fn offer_chain_hashes(s: Span) -> Parsed<Offer> {
    let (s, chs) = many0(chain_hash_be)(s)?;
    Ok((s, Offer::ChainHashes(chs)))
}

pub fn offer_node_id(s: Span) -> Parsed<Offer> {
    let (s, pk) = public_key(s)?;
    Ok((s, Offer::PublicKey(pk)))
}

#[derive(Debug)]
pub enum ScidOrPublicKey {
    Scid(ShortChannelId),
    PublicKey(PublicKey),
}

impl ToValue for ScidOrPublicKey {
    fn to_value(&self) -> Value {
        match self {
            ScidOrPublicKey::Scid(scid) => scid.to_value(),
            ScidOrPublicKey::PublicKey(pk) => pk.to_value(),
        }
    }
}

fn scid(s: Span) -> Parsed<ScidOrPublicKey> {
    let (s, _) = parse(
        verify(u8, |v| *v == 0 || *v == 1),
        ann("Refers to", |v: &u8| {
            if *v == 0 {
                Value::num(0)
            } else if *v == 1 {
                Value::num(1)
            } else {
                unreachable!()
            }
        }),
    )(s)?;
    let (s, scid) = short_channel_id(s)?;
    Ok((s, ScidOrPublicKey::Scid(scid)))
}

fn pk(s: Span) -> Parsed<ScidOrPublicKey> {
    let (s, _) = verify(peek(u8), |v| *v == 2 || *v == 3)(s)?;
    let (s, pk) = parse(public_key, ann("First node ID", auto()))(s)?;
    Ok((s, ScidOrPublicKey::PublicKey(pk)))
}

fn onionmsg_hop(s: Span) -> Parsed<()> {
    let (s, _) = parse(public_key, ann("Blinded node ID", auto()))(s)?;
    let (s, enclen) = parse(be_u16, ann("Length of encrypted data", auto()))(s)?;
    let (s, _) = parse(bytes(enclen), ann("Encrypted data", auto()))(s)?;
    Ok((s, ()))
}

pub fn paths(s: Span) -> Parsed<Offer> {
    let (s, _) = nom::branch::alt((scid, pk))(s)?;
    let (s, _) = parse(public_key, ann("Blinding", auto()))(s)?;
    let (s, _) = parse(
        length_count(
            parse(u8, ann("Number of hops", auto())),
            parse(with("list", "enumerate", onionmsg_hop), ann("Hop", auto())),
        ),
        ann("Hops", Value::Nil),
    )(s)?;
    Ok((s, Offer::Paths))
}

pub fn other(s: Span) -> Parsed<Offer> {
    let (s, bytes) = many0(u8)(s)?;
    Ok((s, Offer::Other(bytes.into())))
}

pub fn tlv_record(s: Span) -> Parsed<Offer> {
    let (s, typ) = parse(u8, ann("Type", auto()))(s)?;
    let (s, length) = parse(u8, ann("Length", auto()))(s)?;

    let (s, value) = parse_slice(
        length,
        parse(
            match typ {
                2 => offer_chain_hashes,
                6 => currency,
                10 => description,
                16 => paths,
                18 => issuer,
                22 => offer_node_id,
                _ => other,
            },
            ann("Value", auto()),
        ),
    )(s)?;

    let annotation = match typ {
        2 => "Offer chains",
        4 => "Offer metadata",
        6 => "Offer currency",
        8 => "Offer amount",
        10 => "Offer description",
        12 => "Offer features",
        14 => "Offer absolute expiry",
        16 => "Offer paths",
        18 => "Offer issuer",
        20 => "Offer quantity max",
        22 => "Offer node ID",
        240 => "Signature",
        _ => "Unknown type",
    };

    Ok((s.with("annotation", annotation), value))
}

pub fn bolt12(s: Span) -> Parsed<String> {
    let (s, records) = parse(
        many0(parse(tlv_record, ann("TLV Record", Value::Nil))),
        ann("TLV Stream", Value::Nil),
    )(s)?;

    Ok((s, format!("{records:?}")))
}
