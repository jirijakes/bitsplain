use bitcoin::absolute::LockTime;

use crate::bitcoin::*;
use crate::btc::datatypes::*;
use crate::dsl::{ann, auto};
use crate::nom::combinator::{peek, success};
use crate::nom::multi::{length_count, many_m_n};
use crate::nom::number::complete::be_u8;
use crate::parse::*;
use crate::tree::Tag;
use crate::types::*;
use crate::value::*;

pub fn out_point(s: Span) -> Parsed<OutPoint> {
    let (s, txid) = parse(
        txid,
        ann("Previous Transaction", auto())
            .doc("ID of transaction, which contains this input as one of its outputs"),
    )(s)?;
    let (s, vout) = parse(
        uint32,
        ann("Output Index", auto())
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
    let (s, script) = parse(output_script, ann("Output Script", Value::Nil))(s)?;

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
    } else if script.is_p2wsh() {
        "P2WSH"
    } else if script.is_p2wpkh() {
        "P2WPKH"
    } else if script.is_p2tr() {
        "P2TR"
    } else if script.is_op_return() {
        "OP_RETURN"
    } else {
        "NSTD"
    };

    let tx_out = TxOut {
        value: value.amount(),
        script_pubkey: script,
    };

    let s = s.add_tag(Tag {
        label: script_type.to_string(),
        color: None,
        doc: None,
    });

    Ok((s, tx_out))
}

/// Parses all transaction outputs.
pub fn tx_outs(input: Span) -> Parsed<Vec<TxOut>> {
    let (s, vout_n) = parse(
        varint,
        ann("Output Count", auto()).doc("Number of outputs of this transaction"),
    )(input)?;
    many_m_n(
        vout_n as usize,
        vout_n as usize,
        parse(with("list", "enumerate", tx_out), ann("vout", Value::Nil)),
    )(s)
}

/// Parses all transaction inputs.
pub fn tx_ins(input: Span) -> Parsed<Vec<TxIn>> {
    let (s, vin_n) = parse(
        varint,
        ann("Input Count", auto()).doc("Number of inputs participating in this transaction"),
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
        ann("Outpoint", |o: &OutPoint| {
            Value::text(format!("{:?}:{}", o.txid, o.vout))
        }),
    )(input)?;
    let (s, scr) = parse(script, ann("Input Script", Value::Nil))(s)?;
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

/// Parser of a script in transaction output.
pub fn output_script(input: Span) -> Parsed<ScriptBuf> {
    let (s, len) = parse(varint, ann("Length", auto()))(input)?;
    let (s, script_data) = parse(
        bytes(len),
        ann("Script Data", |b: &Vec<_>| Value::bytes(b.to_vec())),
    )(s)?;
    let script: ScriptBuf = script_data.into();
    if script.is_witness_program() {
        s.insert(ann(
            "Witness Version",
            match script.witness_version() {
                None => Value::text("(Unknown)"),
                Some(v) => Value::num(v.to_num()),
            },
        ));

        if script.is_p2wpkh() {
            s.insert(ann("Length of Witness Program", Value::Size(20)));
            s.insert(
                ann(
                    "Witness Program",
                    Value::bytes(script.as_bytes()[2..].to_vec()),
                )
                .bip(141)
                .splain("Witness version 0 and 20-byte program indicate P2WPKH output. In P2WPKH output, witness program is a HASH160 hash of public key."),
            );
        } else if script.is_p2wsh() {
            s.insert(ann("Length of Witness Program", Value::Size(32)));
            s.insert(
                ann(
                    "Witness Program",
                    Value::bytes(script.as_bytes()[2..].to_vec()),
                )
                .bip(141)
                .splain("Witness version 0 and 32-byte program indicate P2WSH output. In P2WSH output, witness program is SHA256 hash of script."),
            );
        } else if script.is_p2tr() {
            s.insert(ann("Length of Witness Program", Value::Size(32)));
            s.insert(
                ann(
                    "Witness Program",
                    Value::bytes(script.as_bytes()[2..].to_vec()),
                )
                    .bip(341)
                    .splain("Witness version 1 and 32-byte program indicate P2TR output. In P2TR output, witness program represents public key."),
            );
        };
    }
    // s.insert(ann("Witness Version", Value::Script(script.clone())))
    Ok((s.with("datatype", "script"), script))
}

/// Parse one witness item.
///
// TODO: in the future we might be able to interpret the witnesses
// based on previous output. Once the previous output is available,
// let us to do.
pub fn witness_item(_vin: TxIn) -> impl Fn(Span) -> Parsed<Vec<u8>> {
    move |s: Span| {
        let (s, len) = parse(varint, ann("Length", |n: &u64| Value::Size(*n)))(s)?;
        let (s, w) = parse(
            length_count(success(len), be_u8),
            ann("Witness Data", auto()),
        )(s)?;
        Ok((s, w))
    }
}

/// Parses one witness stack, i. e. all witness items associated with one input.
pub fn witness_stack(vin: TxIn) -> impl Fn(Span) -> Parsed<Vec<Vec<u8>>> {
    move |s: Span| {
        let (s, cnt) = parse(varint, ann("Count", auto()))(s)?;
        length_count(
            success(cnt),
            parse(witness_item(vin.clone()), ann("Witness Item", Value::Nil)),
        )(s)
    }
}

/// Parses complete witness structure.
fn witness_structure(vins: Vec<TxIn>) -> impl Fn(Span) -> Parsed<Vec<Vec<Vec<u8>>>> {
    move |s: Span| {
        s.insert(ann("Length", Value::Size(vins.len() as u64))); // FIXME: Does not render
        let (s, w) = vins.iter().try_fold((s, vec![]), |(s, mut ws), vin| {
            parse(
                with("list", "enumerate", witness_stack(vin.clone())),
                ann("Witness Stack", Value::Nil),
            )(s)
            .map(|(s, w)| {
                ws.push(w);
                (s, ws)
            })
        })?;
        s.insert(ann("Length 2", Value::Size(vins.len() as u64))); // FIXME: Does not render
        Ok((s, w))
    }
}

/// Parse Bitcoin transaction.
pub fn tx(s: Span) -> Parsed<Transaction> {
    // let bm1 = s.bookmark();
    let (s, version) = parse(
        int32,
        ann("Transaction Version", auto())
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
    let version = transaction::Version(version);

    let (s, marker) = peek(be_u8)(s)?;
    let (s, flag) = if marker == 0 {
        let (s, _marker) =
            parse(
                be_u8,
                ann("Marker", auto())
                    .bip(144)
                    .doc("Indicates whether the transaction uses extended serialization. 0 means extended, otherwise not extended (pre-segwit).")
                    .splain(|m: &u8| {
                        let s = if *m == 0 {
                            "Marker 0 indicates extended serialization, i. e. segwit."
                        } else {
                            "Marker other than 0 indicates pre-segwit serialization."
                        };
                        s.to_string()
                    })
            )(s)?;

        parse(
            be_u8,
            ann("Flag", auto())
                .bip(144)
                .doc("Indicates features used by the transaction. Currently only 1 is used.")
                .splain(|f: &u8| {
                    if *f == 1 {
                        "Flag 1 indicates that transaction has witness structure".to_string()
                    } else {
                        format!("Flag {} is non-standard.", f)
                    }
                }),
        )(s)?
    } else {
        (s, 0)
    };
    let bm2 = s.bookmark();
    let (s, mut vin) = parse(tx_ins, ann("Input List", Value::Nil))(s)?;
    let (s, vout) = parse(tx_outs, ann("Output List", Value::Nil))(s)?;

    let (s, witnesses) = if flag == 1 {
        parse(
            witness_structure(vin.clone()),
            ann("Witness Structure", Value::Nil),
        )(s)?
    } else {
        (s, vec![])
    };

    // Assign witnesses to their relevant inputs.
    witnesses
        .into_iter()
        .enumerate()
        .for_each(|(index, witness)| vin[index].witness = witness.into());

    let (s, locktime) = parse(
        uint32,
        ann("Lock Time", auto())
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

    let total = vout
        .iter()
        .fold(0u128, |acc, v| acc + u128::from(v.value.to_sat()));
    let tx = Transaction {
        version,
        lock_time: LockTime::from_consensus(locktime),
        input: vin,
        output: vout,
    };
    s.insert_at(
        &bm2,
        ann("Txid", Value::Hash(tx.txid().to_raw_hash()))
            .doc("ID of this transaction as defined pre-segwit."),
    );
    s.insert_at(
        &bm2,
        ann("Wtxid", Value::Hash(tx.wtxid().to_raw_hash()))
            .doc("Segwit-aware ID of this transaction."),
    );
    s.insert_at(&bm2, ann("Size", Value::Size(tx.base_size() as u64)));
    s.insert_at(&bm2, ann("Vsize", Value::Size(tx.vsize() as u64)));
    s.insert_at(&bm2, ann("Weight", Value::Size(tx.weight().to_wu())));
    s.insert_at(
        &bm2,
        ann("Total amount", Value::Sat(Sat::new(total)))
            .doc("Sum of amounts of all outputs of this transaction"),
    );
    Ok((s, tx))
}
