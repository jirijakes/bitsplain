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

pub fn channel_update(s: Span) -> Parsed<()> {
    let (s, _) = value(258, be_u16)(s)?;
    let (s, _) = parse(signature, ann("Signature", auto()))(s)?;
    let (s, _) = parse(chain_hash_be, ann("Chain hash", auto()))(s)?;
    let (s, _) = parse(short_channel_id, ann("Short channel ID", auto()))(s)?;
    let (s, _) = parse(timestamp(be_u32), ann("Timestamp", auto()))(s)?;
    let (s, _) = parse(
        flags(
            u8,
            &[
                (0, ann("must_be_one", auto()).doc("Ignored. It used to indicate presence of 'HTLC maximum msat' field, however nowadays it is always present.")),
                (1, ann("dont_forward", auto()).doc("Indicates whether channel has been announced or not yet and thus whether this update may be forwarded.")),
            ],
        ),
        ann("Message flags", auto()),
    )(s)?;
    let (s, _) = parse(
        flags(
            u8,
            &[
                (
                    0,
                    ann("direction", auto())
                        .doc("Direction this update refers to.")
                        .splain(|dir: &bool| {
                            if *dir {
                                String::from("1 (true): node_id_2 is originator of the message.")
                            } else {
                                String::from("0 (false): node_id_1 is originator of the message.")
                            }
                        }),
                ),
                (
                    1,
                    ann("disable", auto())
                        .doc("Whether the channel should be temporarily disabled.")
                        .splain(|disable: &bool| {
                            if *disable {
                                String::from("1 (true): channel should be disabled.")
                            } else {
                                String::from("0 (false): channel should not be disabled.")
                            }
                        }),
                ),
            ],
        ),
        ann("Channel flags", auto()),
    )(s)?;
    let (s, _) = parse(
        be_u16,
        ann("CLTV expiry delta", auto())
            .doc("Number of blocks to substract from incoming HTLCs' cltv_expiry."),
    )(s)?;
    let (s, _) = parse(
        be_u64,
        ann("HTLC minimum msat", auto())
            .doc("Minimum HTLC value in millisatoshi that the channel peer will accept."),
    )(s)?;
    let (s, _) = parse(be_u32, ann("Fee base msat", auto()))(s)?;
    let (s, _) = parse(be_u32, ann("Fee proportional millionths", auto()))(s)?;
    let (s, _) = parse(
        be_u64,
        ann("HTLC maximum msat", auto()).doc(
            "Maximum value in millisatoshi that the channel peer will send for a single HTLC.",
        ),
    )(s)?;
    Ok((s, ()))
}

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
