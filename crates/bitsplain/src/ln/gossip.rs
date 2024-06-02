use lightning::ln::features::{ChannelFeatures, NodeFeatures};
use lightning::ln::msgs::*;

use crate::dsl::{ann, auto};
use crate::ln::{rgb_color, short_channel_id};
use crate::nom::combinator::{success, value};
use crate::nom::multi::length_count;
use crate::nom::number::complete::*;
use crate::parse::*;
use crate::types::*;
use crate::value::Value;

// pub fn gossip_timestamp_filter(s: Span) -> Parsed<GossipTimestampFilter> {
//     let (s, _) = value(265, be_u16)(s)?;
//     let (s, chain_hash) = parse(chain_hash_be, ann("Chain hash", auto()))(s)?;
//     let (s, first_timestamp) = parse(timestamp(be_u32), ann("First timestamp", auto()))(s)?;
//     let (s, timestamp_range) = parse(be_u32, ann("Timestamp range", auto()))(s)?;

//     Ok((
//         s,
//         GossipTimestampFilter {
//             chain_hash: chain_hash.block_hash,
//             first_timestamp: first_timestamp.unix_timestamp() as u32,
//             timestamp_range,
//         },
//     ))
// }

pub fn node_announcement(s: Span) -> Parsed<()> {
    let (s, _) = value(257, be_u16)(s)?;
    let (s, _signature) = parse(signature, ann("Signature", auto()))(s)?;
    let (s, len) = parse(be_u16, ann("Features length", auto()))(s)?;

    let (s, features) = parse(
        length_count(success(len), u8),
        ann("Features", |b: &Vec<u8>| Value::bytes(b.clone())),
    )(s)?;

    let _features = NodeFeatures::from_le_bytes({
        let mut b = features;
        b.reverse();
        b
    });

    let (s, _timestamp) = parse(timestamp(be_u32), ann("Timestamp", auto()))(s)?;
    let (s, _node_id) = parse(public_key, ann("Node ID", auto()))(s)?;
    let (s, _rgb_color) = parse(rgb_color, ann("RGB Color", auto()))(s)?;
    let (s, _alias) = parse(
        bytes(32usize),
        ann("Alias", |b: &Vec<u8>| {
            Value::text(String::from_utf8_lossy(b).trim_end_matches('\0'))
        }),
    )(s)?;
    let (s, addr_len) = parse(be_u16, ann("Addresses length", auto()))(s)?;
    let (s, _addresses) = parse(
        length_count(success(addr_len), u8),
        ann("Addresses", "TODO"),
    )(s)?;

    Ok((s, ()))
}

macro_rules! p {
    ($parser: expr, $ann: literal) => {
        parse($parser, ann($ann, auto()))
    };
}

macro_rules! ins {
    ($parser: expr => $ann: literal) => {
        p!($parser, $ann)
    };

    ($expr: expr) => {
        $expr
    };
}

macro_rules! parser {
    ($name: ident, $(($($item:tt)*)),+ ) => {
        pub fn $name(s: Span) -> Parsed<()> {
            $(let (s, _) = ins!($($item)*)(s)?;)+
                Ok((s, ()))
        }
    };
}

macro_rules! flag8 {
    ($($idx: literal => $ann: literal),+) => {
        flags(u8, &[$(($idx, ann($ann, auto()))),+])
    };
}

// TODO: Tests from eclair / non-regression on channel_update
parser!(channel_update,
        (value(258, be_u16)),
        (signature => "Signature"),
        (chain_hash_be => "Chain hash"),
        (short_channel_id => "Short channel ID"),
        (timestamp(be_u32) => "Timestamp"),
        (flag8!(0 => "must_be_one", 1 => "dont_forward") => "Message flags"),
        (flag8!(0 => "direction", 1 => "disable") => "Channel flags"),
        (be_u16 => "CLTV expiry delta"),
        (be_u64 => "HTLC minimum msat"),
        (be_u32 => "Fee base msat"),
        (be_u32 => "Fee proportional millionths"),
        (be_u64 => "HTLC maximum msat")
);

pub fn channel_announcement(s: Span) -> Parsed<()> {
    let (s, _) = value(256, be_u16)(s)?;
    let (s, _node_signature_1) = parse(signature, ann("Node signature 1", auto()))(s)?;
    let (s, _node_signature_2) = parse(signature, ann("Node signature 2", auto()))(s)?;
    let (s, _bitcoin_signature_1) = parse(signature, ann("Bitcoin signature 1", auto()))(s)?;
    let (s, _bitcoin_signature_2) = parse(signature, ann("Bitcoin signature 2", auto()))(s)?;
    let (s, len) = parse(be_u16, ann("Features length", auto()))(s)?;

    let (s, features) = parse(
        length_count(success(len), u8),
        ann("Features", |b: &Vec<u8>| Value::bytes(b.to_vec())),
    )(s)?;

    let _features = ChannelFeatures::from_le_bytes({
        let mut b = features;
        b.reverse();
        b
    });

    // TODO: print interpeted features

    let (s, _chain_hash) = parse(chain_hash_be, ann("Chain hash", auto()))(s)?;
    let (s, _scid) = parse(short_channel_id, ann("Short channel ID", auto()))(s)?;
    let (s, _node_id_1) = parse(public_key, ann("Node 1 ID", auto()))(s)?;
    let (s, _node_id_2) = parse(public_key, ann("Node 2 ID", auto()))(s)?;
    let (s, _bitcoin_key_1) = parse(public_key, ann("Bitcoin key 1", auto()))(s)?;
    let (s, _bitcoin_key_2) = parse(public_key, ann("Bitcoin key 2", auto()))(s)?;

    Ok((s, ()))
}
