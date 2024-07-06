use bytes::Bytes;
use nom::bytes::complete::tag;
use nom::combinator::{cond, peek, recognize, value, verify};
use nom::multi::length_count;
use nom::sequence::preceded;
use nom::InputTake;

use crate::bitcoin::PublicKey;
use crate::dsl::{ann, auto};
use crate::nom::combinator::success;
use crate::nom::multi::{length_value, many0};
use crate::nom::number::complete::*;
use crate::parse::*;
use crate::types::*;
use crate::value::{ToValue, Value};

#[derive(Debug, Clone)]
pub enum Offer {
    ChainHashes(Vec<ChainHash>),
    Description(String),
    Issuer(String),
    Currency(String),
    Paths(Vec<String>),
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
            Offer::Paths(ps) => Value::text(format!("{:?}", ps)),
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
    Scid(Vec<u8>),
    PublicKey(PublicKey),
}

fn scid(s: Span) -> Parsed<ScidOrPublicKey> {
    let (s, dir) = parse(verify(u8, |v| *v == 0 || *v == 1), ann("xxx", auto()))(s)?;
    let (s, scid) = bytes(9usize)(s)?;
    Ok((s, ScidOrPublicKey::Scid(scid)))
}

fn pk(s: Span) -> Parsed<ScidOrPublicKey> {
    let (s, _) = verify(peek(u8), |v| *v == 2 || *v == 3)(s)?;
    let (s, pk) = parse(public_key, ann("XXX", auto()))(s)?;
    Ok((s, ScidOrPublicKey::PublicKey(pk)))
}

fn onionmsg_hop(s: Span) -> Parsed<(PublicKey, Vec<u8>)> {
    let (s, blinded_node_id) = parse(public_key, ann("prdel", auto()))(s)?;
    let (s, enclen) = be_u16(s)?;
    let (s, data) = bytes(enclen)(s)?;
    Ok((s, (blinded_node_id, data)))
}

pub fn paths(s: Span) -> Parsed<Offer> {
    let (s, scid_or_pk) = nom::branch::alt((scid, pk))(s)?;
    let (s, blinding) = parse(public_key, ann("YYY", Value::text("aaaa")))(s)?;
    let (s, hops) = parse(
        length_count(u8, parse(onionmsg_hop, ann("cibul", "voni"))),
        ann("hovno", "smrdi"),
    )(s)?;
    Ok((s, Offer::Paths(vec![])))
}

pub fn other(s: Span) -> Parsed<Offer> {
    let (s, bytes) = many0(u8)(s)?;
    Ok((s, Offer::Other(bytes.into())))
}

pub fn tlv_record(s: Span) -> Parsed<Offer> {
    let (s, typ) = parse(u8, ann("Type", auto()))(s)?;
    let (s, length) = parse(u8, ann("Length", auto()))(s)?;
    let (s, rest) = s.take_split(length.into());

    println!("\n1. {:?}\n", s);
    println!("\n\n2. {:?}\n\n", rest);

    let (s, value) = parse(paths, ann("Prd", "xxx"))(s)?;

    println!("\n3. {:?}\n", s);

    // let (s, value) = parse(
    // length_value(
    //     success(length),
    //     parse(paths, ann("Prd", "xxx"))// match typ {
    //            // 2 => offer_chain_hashes,
    //            // 6 => currency,
    //            // 10 => description,
    //            // 16 => ,
    //            // 18 => issuer,
    //            // 22 => offer_node_id,
    //            // _ => unimplemented!() // parse(other, ann("???", Value::text("..."))),
    //            // },
    // ),
    // ann("ZZZ", Value::text("UUU")),
    // )(s)?;

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
