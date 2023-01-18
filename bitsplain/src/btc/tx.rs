use crate::ann::ann;
use crate::ann::auto;
use crate::basic::*;
use crate::bitcoin::*;
use crate::btc::datatypes::*;
use crate::datatypes::Sat;
use crate::nom::combinator::peek;
use crate::nom::multi::length_count;
use crate::nom::multi::many_m_n;
use crate::nom::number::complete::be_u16;
use crate::nom::number::complete::be_u8;
use crate::nom::IResult;
use crate::parse::*;
use crate::tree::Tag;
use crate::value::*;
use crate::Span;

pub fn out_point(s: Span) -> IResult<Span, OutPoint> {
    let (s, txid) = p(
        txid,
        ann("Previous transaction", auto())
            .doc("ID of transaction, which contains this input as one of its outputs"),
    )(s)?;
    let (s, vout) = p(
        uint32,
        ann("Output", auto())
            .doc("Zero-based index pointing to the specific output of previous transaction, which is this input"),
    )(s)?;
    Ok((s, OutPoint { txid, vout }))
}

pub fn tx_out(s: Span) -> IResult<Span, TxOut> {
    let (s, value) = p(
        sat,
        ann("Amount", auto()).doc("Amount of BTC being transfered in this output"),
    )(s)?;
    let bm = s.bookmark();
    let (s, script) = p(script, ann("Script", Value::Nil))(s)?;

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

pub fn tx_outs(input: Span) -> IResult<Span, Vec<TxOut>> {
    let (s, vout_n) = p(
        varint,
        ann("Outputs", auto()).doc("Number of outputs of this transaction"),
    )(input)?;
    many_m_n(
        vout_n as usize,
        vout_n as usize,
        p(with("list", "enumerate", tx_out), ann("vout", Value::Nil)),
    )(s)
}

pub fn tx_ins(input: Span) -> IResult<Span, Vec<TxIn>> {
    let (s, vin_n) = p(
        varint,
        ann("vin_n", auto()).doc("Number of inputs participating in this transaction"),
    )(input)?;
    many_m_n(
        vin_n as usize,
        vin_n as usize,
        p(with("list", "enumerate", tx_in), ann("vin", Value::Nil)),
    )(s)
}

pub fn tx_in(input: Span) -> IResult<Span, TxIn> {
    let (s, out) = p(
        out_point,
        ann("out_point", |o: &OutPoint| {
            Value::String(format!("{:?}:{}", o.txid, o.vout))
        }),
    )(input)?;
    let (s, scr) = p(script, ann("script", Value::Nil))(s)?;
    let (s, (seq, _)) = p(
        alt(uint32, bytes(4u32)),
        ann("Sequence", |(s, bin): &(u32, Vec<u8>)| {
            Value::Alt(
                Box::new(Value::Num(*s as i128)),
                Box::new(Value::Bytes(bin.clone())),
            )
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
pub fn tx(s: Span) -> IResult<Span, Transaction> {
    // let bm1 = s.bookmark();
    let (s, version) = p(
        int32,
        ann("Version", auto())
            .doc("Version, haha")
            .tag(|v: &_| Tag {
                label: format!("V{v}"),
                color: None,
                doc: None,
            })
            .splain(|v: &_| format!("Version {v}")),
    )(s)?;
    let (s, flags) = peek(be_u16)(s)?;
    let (s, flags) = if flags == 1 {
        p(be_u16, ann("Flags", auto()))(s)?
    } else {
        (s, 0)
    };
    let bm2 = s.bookmark();
    let (s, vin) = p(tx_ins, ann("Vins", Value::Nil))(s)?;
    let (s, vout) = p(tx_outs, ann("Vouts", Value::Nil))(s)?;

    let (s, _witnesses) = if flags == 1 {
        p(
            length_count(varint, length_count(varint, be_u8)),
            ann("Witnesses", "Witness data"),
        )(s)?
    } else {
        (s, vec![])
    };

    let (s, locktime) = p(uint32, ann("Locktime", auto()))(s)?;

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
            .doc("ID of this transaction as defined pre-segwit"),
    );
    s.insert_at(
        &bm2,
        ann("Wtxid", Value::Hash(tx.wtxid().as_hash())).doc("Segwit-aware ID of this transaction"),
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
