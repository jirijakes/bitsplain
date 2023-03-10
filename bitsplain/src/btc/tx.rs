use crate::bitcoin::*;
use crate::btc::datatypes::*;
use crate::dsl::{ann, auto};
use crate::nom::combinator::peek;
use crate::nom::multi::{length_count, many_m_n};
use crate::nom::number::complete::{be_u16, be_u8};
use crate::parse::*;
use crate::tree::Tag;
use crate::types::*;
use crate::value::*;

pub fn out_point(s: Span) -> Parsed<OutPoint> {
    let (s, txid) = parse(
        txid,
        ann("Previous transaction", auto())
            .doc("ID of transaction, which contains this input as one of its outputs"),
    )(s)?;
    let (s, vout) = parse(
        uint32,
        ann("Output", auto())
            .doc("Zero-based index pointing to the specific output of previous transaction, which is this input"),
    )(s)?;
    Ok((s, OutPoint { txid, vout }))
}

pub fn tx_out(s: Span) -> Parsed<TxOut> {
    let (s, value) = parse(
        sat,
        ann("Amount", auto()).doc("Amount of BTC being transfered in this output"),
    )(s)?;
    let bm = s.bookmark();
    let (s, script) = parse(script, ann("Script", Value::Nil))(s)?;

    //    s.insert_before(
    s.insert_at(
        &bm,
        ann(
            "Address",
            Value::Addr(Address::from_script(&script, Network::Bitcoin).ok()),
        ),
    );

    let script_type = if script.is_p2pk() {
        "P2PK"
    } else if script.is_p2sh() {
        "P2SH"
    } else if script.is_p2pkh() {
        "P2PKH"
    } else if script.is_v0_p2wsh() {
        "P2WSH"
    } else if script.is_v0_p2wpkh() {
        "P2WPKH"
    } else if script.is_v1_p2tr() {
        "P2TR"
    } else if script.is_op_return() {
        "OP_RETURN"
    } else {
        "NSTD"
    };

    let tx_out = TxOut {
        value: value.sat(),
        script_pubkey: script,
    };

    let s = s.add_tag(Tag {
        label: script_type.to_string(),
        color: None,
        doc: None,
    });

    Ok((s, tx_out))
}

pub fn tx_outs(input: Span) -> Parsed<Vec<TxOut>> {
    let (s, vout_n) = parse(
        varint,
        ann("Outputs", auto()).doc("Number of outputs of this transaction"),
    )(input)?;
    many_m_n(
        vout_n as usize,
        vout_n as usize,
        parse(with("list", "enumerate", tx_out), ann("vout", Value::Nil)),
    )(s)
}

pub fn tx_ins(input: Span) -> Parsed<Vec<TxIn>> {
    let (s, vin_n) = parse(
        varint,
        ann("vin_n", auto()).doc("Number of inputs participating in this transaction"),
    )(input)?;
    many_m_n(
        vin_n as usize,
        vin_n as usize,
        parse(with("list", "enumerate", tx_in), ann("vin", Value::Nil)),
    )(s)
}

pub fn tx_in(input: Span) -> Parsed<TxIn> {
    let (s, out) = parse(
        out_point,
        ann("out_point", |o: &OutPoint| {
            Value::text(format!("{:?}:{}", o.txid, o.vout))
        }),
    )(input)?;
    let (s, scr) = parse(script, ann("script", Value::Nil))(s)?;
    let (s, (seq, _)) = parse(
        alt(uint32, bytes(4u32)),
        ann("Sequence", |(s, bin): &(u32, Vec<u8>)| {
            Value::alt(Value::Num(*s as i128), Value::bytes(bin.clone()))
        }),
    )(s)?;
    Ok((
        s,
        TxIn {
            previous_output: out,
            script_sig: scr,
            sequence: Sequence(seq),
            witness: Witness::new(),
        },
    ))
}

/// Parse Bitcoin transaction.
pub fn tx(s: Span) -> Parsed<Transaction> {
    // let bm1 = s.bookmark();
    let (s, version) = parse(
        int32,
        ann("Version", auto())
            .doc("Version number of transaction format indicating which set of rules should be used for validation. Currently only 1 and 2 are standard.")
            .splain(|v: &_| {
                let s = if *v == 1 {
                    "Version 1 indicates original version without any additional features."
                } else if *v == 2 {
                    "Version 2 allows to use OP_CHECKSEQUENCEVERIFY."
                } else {
                    "Non-standard version."
                };
                s.to_string()
            })
            .bip(68),
    )(s)?;
    let (s, flags) = peek(be_u16)(s)?;
    let (s, flags) = if flags == 1 {
        parse(be_u16, ann("Flags", auto()))(s)?
    } else {
        (s, 0)
    };
    let bm2 = s.bookmark();
    let (s, vin) = parse(tx_ins, ann("Vins", Value::Nil))(s)?;
    let (s, vout) = parse(tx_outs, ann("Vouts", Value::Nil))(s)?;

    let (s, _witnesses) = if flags == 1 {
        parse(
            length_count(varint, length_count(varint, be_u8)),
            ann("Witnesses", "Witness data"),
        )(s)?
    } else {
        (s, vec![])
    };

    let (s, locktime) = parse(
        uint32,
        ann("Locktime", auto())
            .doc("Earliest time the transaction can be mined in to a block.")
            .splain(|cons: &u32| {
                if *cons == 0 {
                    "Locktime 0 = no locking".to_string()
                } else {
                    match LockTime::from_consensus(*cons) {
                        LockTime::Blocks(b) => format!(
                            "Locktime < 500,000,000: transaction is unlocked at block height {}.",
                            b
                        ),
                        LockTime::Seconds(s) => format!(
                            "Locktime >= 500,000,000: transaction is unlocked at unix time {}, i. e. on {}.",
                            s,
                            time::OffsetDateTime::from_unix_timestamp(s.to_consensus_u32().into()).unwrap()
                        ),
                    }
                }
            })
    )(s)?;

    let total = vout.iter().fold(0, |acc, v| acc + v.value);
    let tx = Transaction {
        version,
        lock_time: PackedLockTime(locktime),
        input: vin,
        output: vout,
    };
    s.insert_at(
        &bm2,
        ann("Txid", Value::Hash(tx.txid().as_hash()))
            .doc("ID of this transaction as defined pre-segwit."),
    );
    s.insert_at(
        &bm2,
        ann("Wtxid", Value::Hash(tx.wtxid().as_hash())).doc("Segwit-aware ID of this transaction."),
    );
    s.insert_at(&bm2, ann("Size", Value::Size(tx.size() as u64)));
    s.insert_at(&bm2, ann("Vsize", Value::Size(tx.vsize() as u64)));
    s.insert_at(&bm2, ann("Weight", Value::Size(tx.weight() as u64)));
    s.insert_at(
        &bm2,
        ann("Total amount", Value::Sat(Sat::new(total)))
            .doc("Sum of amounts of all outputs of this transaction"),
    );
    Ok((s, tx))
}
