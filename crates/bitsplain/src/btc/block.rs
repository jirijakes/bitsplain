use bitcoin::block::Version;
use bitcoin::hash_types::TxMerkleNode;
use bitcoin::hashes::Hash;
use bitcoin::*;

use crate::dsl::{ann, auto};
use crate::parse::*;
use crate::types::*;
use crate::value::Value;
use crate::*;

pub fn block_header(s: Span) -> Parsed<()> {
    let (s, (_, version)) = parse(
        alt(bytes_be(4u32), int32),
        ann("Version", auto())
            .bip(9)
            .www("https://scribe.rip/fcats-blockchain-incubator/understanding-the-bitcoin-blockchain-header-a2b0db06b515")
            .www("https://rusty.ozlabs.org/2016/04/01/bip9-versionbits-in-a-nutshell.html")
            .doc("Number that provides means for the miners to signal readiness for soft forks.")
            .splain(splain_version),
    )(s)?;

    let bm = s.bookmark();

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

    let (s, (_, bits)) = parse(
        alt(bytes_be(4u32), uint32),
        ann("Bits", auto()).doc("Compact form of current target."),
    )(s)?;

    let (s, (_, nonce)) = parse(
        alt(bytes_be(4u32), uint32),
        ann("Nonce", auto())
            .www("https://en.bitcoin.it/wiki/Nonce")
            .doc(
                "32-bit value that is repeatedly adjusted during mining process in order to meet consensus requirements, i.e. hash of the block must be less or equal to current target of the network."
            ))(s)?;

    let block_header = block::Header {
        version: Version::from_consensus(version),
        prev_blockhash: BlockHash::from_raw_hash(prev_blockhash),
        merkle_root: TxMerkleNode::from_raw_hash(merkle_root),
        time: time.unix_timestamp() as u32,
        bits: CompactTarget::from_consensus(bits),
        nonce,
    };

    s.insert_at(
        &bm,
        ann(
            "Block hash",
            Value::Hash(block_header.block_hash().to_raw_hash()),
        )
        .doc("Hash of this block calculated as SHA256(SHA256(serialized header))."),
    );

    // TODO: The condition is here only to ensure that parsing of non-block header data does not fail.
    // Might be improved by preconditions or other sort of validations.
    if block_header.target().difficulty(Network::Bitcoin) > 0 {
        s.insert(
            ann(
                "Difficulty",
                Value::Num(i128::try_from(block_header.difficulty(Network::Bitcoin)).unwrap()),
            )
            .www("https://en.bitcoin.it/wiki/Difficulty")
            .doc("Measure of how difficult it is to find a hash below a given target. "),
        );
    };

    s.insert(
        ann("Target", Value::display(block_header.target()))
            .www("https://en.bitcoin.it/wiki/Target")
            .doc("Value below which must be a valid block header hash.")
            .splain({
                let as_num = num_bigint::BigUint::from_bytes_le(block_header.block_hash().to_byte_array().as_slice());
                format!(
                    "Hash of this header {} can be interpreted as number {}, which must be equal or less than this target.",
                    block_header.block_hash(),
                    as_num
                )
            }),
    );

    s.insert(
        ann("Work", Value::display(block_header.work())).doc("Work that this block contributes."),
    );

    Ok((s, ()))
}

/// Generate splain for block header version.
fn splain_version(v: &(Vec<u8>, i32)) -> String {
    let version = Version::from_consensus(v.1);
    signaling_for(&version)
}

fn signaling_for(version: &Version) -> String {
    let csv = if version.is_signalling_soft_fork(0) {
        "CSV (bit 0 is active)"
    } else {
        ""
    };

    let segwit = if version.is_signalling_soft_fork(1) {
        "Segwit (bit 1 is active)"
    } else {
        ""
    };

    let taproot = if version.is_signalling_soft_fork(2) {
        "Taproot (bit 2 is active)"
    } else {
        ""
    };

    let all = vec![csv, segwit, taproot]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();

    if all.is_empty() {
        "Miner is not signaling for any soft fork (no bit is active).".to_string()
    } else {
        format!("Miner is signaling for {}.", all.join(", "))
    }
}
