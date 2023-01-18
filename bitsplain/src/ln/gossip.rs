use lightning::ln::features::{ChannelFeatures, NodeFeatures};
use lightning::ln::msgs::*;

use crate::ann::{ann, auto};
use crate::ln::{rgb_color, short_channel_id};
use crate::nom::combinator::{success, value};
use crate::nom::multi::length_count;
use crate::nom::number::complete::*;
use crate::nom::IResult;
use crate::parse::*;
use crate::types::*;
use crate::value::Value;
use crate::Span;

pub fn gossip_timestamp_filter(s: Span) -> IResult<Span, GossipTimestampFilter> {
    let (s, _) = value(265, be_u16)(s)?;
    let (s, chain_hash) = p(chain_hash, ann("Chain hash", auto()))(s)?;
    let (s, first_timestamp) = p(timestamp(be_u32), ann("First timestamp", auto()))(s)?;
    let (s, timestamp_range) = p(be_u32, ann("Timestamp range", auto()))(s)?;

    Ok((
        s,
        GossipTimestampFilter {
            chain_hash: chain_hash.block_hash,
            first_timestamp: first_timestamp.unix_timestamp() as u32,
            timestamp_range,
        },
    ))
}

pub fn node_announcement(s: Span) -> IResult<Span, ()> {
    let (s, _) = value(257, be_u16)(s)?;
    let (s, _signature) = p(signature, ann("Signature", auto()))(s)?;
    let (s, len) = p(be_u16, ann("Features length", auto()))(s)?;

    let (s, features) = p(
        length_count(success(len), u8),
        ann("Features", |b: &Vec<u8>| Value::bytes(b.clone())),
    )(s)?;

    let _features = NodeFeatures::from_le_bytes({
        let mut b = features;
        b.reverse();
        b
    });

    let (s, _timestamp) = p(timestamp(be_u32), ann("Timestamp", auto()))(s)?;
    let (s, _node_id) = p(public_key, ann("Node ID", auto()))(s)?;
    let (s, _rgb_color) = p(rgb_color, ann("RGB Color", auto()))(s)?;
    let (s, _alias) = p(
        bytes(32usize),
        ann("Alias", |b: &Vec<u8>| {
            Value::text(String::from_utf8_lossy(b).trim_end_matches('\0'))
        }),
    )(s)?;
    let (s, addr_len) = p(be_u16, ann("Addresses length", auto()))(s)?;
    let (s, _addresses) = p(
        length_count(success(addr_len), u8),
        ann("Addresses", "TODO"),
    )(s)?;

    Ok((s, ()))
}

pub fn channel_update(s: Span) -> IResult<Span, ()> {
    let (s, _) = value(258, be_u16)(s)?;
    let (s, _signature) = p(signature, ann("Signature", auto()))(s)?;
    let (s, _chain_hash) = p(chain_hash, ann("Chain hash", auto()))(s)?;
    let (s, _scid) = p(short_channel_id, ann("Short channel ID", auto()))(s)?;
    let (s, _timestamp) = p(timestamp(be_u32), ann("Timestamp", auto()))(s)?;
    let (s, _) = p(u8, ann("Message flags", auto()))(s)?;
    let (s, _flags) = p(u8, ann("Channel flags", auto()))(s)?;
    let (s, _cltv_expiry_delta) = p(be_u16, ann("CLTV expiry delta", auto()))(s)?;
    let (s, _htlc_minimum_msat) = p(be_u64, ann("HTLC minimum msat", auto()))(s)?;
    let (s, _fee_base_msat) = p(be_u32, ann("Fee base msat", auto()))(s)?;
    let (s, _fee_proportional_millionths) =
        p(be_u32, ann("Fee proportional millionths", auto()))(s)?;
    let (s, _htlc_maximum_msat) = p(be_u64, ann("HTLC maximum msat", auto()))(s)?;

    Ok((s, ()))
}

pub fn channel_announcement(s: Span) -> IResult<Span, ()> {
    let (s, _) = value(256, be_u16)(s)?;
    let (s, _node_signature_1) = p(signature, ann("Node signature 1", auto()))(s)?;
    let (s, _node_signature_2) = p(signature, ann("Node signature 2", auto()))(s)?;
    let (s, _bitcoin_signature_1) = p(signature, ann("Bitcoin signature 1", auto()))(s)?;
    let (s, _bitcoin_signature_2) = p(signature, ann("Bitcoin signature 2", auto()))(s)?;
    let (s, len) = p(be_u16, ann("Features length", auto()))(s)?;

    let (s, features) = p(
        length_count(success(len), u8),
        ann("Features", |b: &Vec<u8>| Value::bytes(b.to_vec())),
    )(s)?;

    let _features = ChannelFeatures::from_le_bytes({
        let mut b = features;
        b.reverse();
        b
    });

    // TODO: print interpeted features

    let (s, _chain_hash) = p(chain_hash, ann("Chain hash", auto()))(s)?;
    let (s, _scid) = p(short_channel_id, ann("Short channel ID", auto()))(s)?;
    let (s, _node_id_1) = p(public_key, ann("Node 1 ID", auto()))(s)?;
    let (s, _node_id_2) = p(public_key, ann("Node 2 ID", auto()))(s)?;
    let (s, _bitcoin_key_1) = p(public_key, ann("Bitcoin key 1", auto()))(s)?;
    let (s, _bitcoin_key_2) = p(public_key, ann("Bitcoin key 2", auto()))(s)?;

    Ok((s, ()))
}
