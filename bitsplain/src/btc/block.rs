use bitcoin::*;

use crate::dsl::{ann, auto};
use crate::parse::*;
use crate::types::*;
use crate::value::Value;
use crate::*;

pub fn block_header(s: Span) -> IResult<Span, BlockHeader> {
    let (s, version) = parse(int32, ann("Version", auto()))(s)?;
    let (s, prev_blockhash) = parse(sha256d, ann("Previous block hash", auto()))(s)?;
    let (s, merkle_root) = parse(sha256d, ann("Merkle root", auto()))(s)?;
    let (s, time) = parse(timestamp(uint32), ann("Timestamp", auto()))(s)?;
    let (s, bits) = parse(uint32, ann("Bits", auto()))(s)?;
    let (s, nonce) = parse(uint32, ann("Nonce", auto()))(s)?;

    let block_header = BlockHeader {
        version,
        prev_blockhash: BlockHash::from_hash(prev_blockhash),
        merkle_root: TxMerkleNode::from_hash(merkle_root),
        time: time.unix_timestamp() as u32,
        bits,
        nonce,
    };

    // TODO: The condition is here only to ensure that parsing of non-block header data does not fail.
    // Might be improved by preconditions or other sort of validations.
    if block_header.target().bits() > 0 {
        s.insert(ann(
            "Difficulty",
            Value::Num(i128::from(block_header.difficulty(Network::Bitcoin))),
        ));
    };

    s.insert(ann("Target", Value::display(block_header.target())));
    s.insert(ann("Work", Value::display(block_header.work())));

    Ok((s, block_header))
}
