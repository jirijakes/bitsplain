use bitcoin::*;

use crate::dsl::{ann, auto};
use crate::parse::*;
use crate::types::*;
use crate::value::Value;
use crate::*;

pub fn block_header(s: Span) -> IResult<Span, ()> {
    let (s, version) = parse(
        int32,
        ann("Version", auto())
            .doc("Number that provides means for the miners to signal readiness for soft forks."),
    )(s)?;
    let (s, prev_blockhash) = parse(
        sha256d,
        ann("Previous block hash", auto()).doc("Hash of the previous block header."),
    )(s)?;
    let (s, merkle_root) = parse(
        sha256d,
        ann("Merkle root", auto())
            .doc("Hash of the root of merkle tree of all transactions within this block."),
    )(s)?;
    let (s, time) = parse(
        timestamp(uint32),
        ann("Timestamp", auto())
            .www("https://en.bitcoin.it/wiki/Block_timestamp")
            .doc("Time of production of the block. It is not supposed to be accurate, its accuracy is in order of one or two hours. It serves to add variation for the block hash and to contribute to safety of the block chain."),
    )(s)?;
    let (s, bits) = parse(
        uint32,
        ann("Bits", auto()).doc("Compact form of current target."),
    )(s)?;
    let (s, nonce) = parse(
        uint32,
        ann("Nonce", auto())
            .www("https://en.bitcoin.it/wiki/Nonce")
            .doc(
                "32-bit value that is repeatedly adjusted during mining process in order to meet consensus requirements (hash of the block less or equal to current target of the network)."
            ))(s)?;

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
        s.insert(
            ann(
                "Difficulty",
                Value::Num(i128::from(block_header.difficulty(Network::Bitcoin))),
            )
            .www("https://en.bitcoin.it/wiki/Difficulty")
            .doc("Measure of how difficult it is to find a hash below a given target. "),
        );
    };

    s.insert(
        ann("Target", Value::display(block_header.target()))
            .www("https://en.bitcoin.it/wiki/Target")
            .doc("Value below which must be a valid block header hash.")
            .splain(format!(
                "Hash of this header {} is less than this target.",
                block_header.block_hash()
            )),
    );
    s.insert(
        ann("Work", Value::display(block_header.work())).doc("Work that this block contributes."),
    );

    Ok((s, ()))
}
