use bitcoin::*;

use crate::ann::ann;
use crate::ann::auto;
use crate::basic::*;
use crate::parse::*;
use crate::value::Value;
use crate::*;

pub fn block_header(s: Span) -> IResult<Span, BlockHeader> {
    let (s, version) = p(int32, ann("Version", auto()))(s)?;
    let (s, prev_blockhash) = p(sha256d, ann("Previous block hash", auto()))(s)?;
    let (s, merkle_root) = p(sha256d, ann("Merkle root", auto()))(s)?;
    let (s, time) = p(timestamp(uint32), ann("Timestamp", auto()))(s)?;
    let (s, bits) = p(uint32, ann("Bits", auto()))(s)?;
    let (s, nonce) = p(uint32, ann("Nonce", auto()))(s)?;

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

    s.insert(ann(
        "Target",
        Value::String(block_header.target().to_string()),
    ));
    s.insert(ann("Work", Value::String(block_header.work().to_string())));

    Ok((s, block_header))
}
