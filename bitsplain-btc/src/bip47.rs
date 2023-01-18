use bitsplain_core::ann::{ann, auto};
use bitsplain_core::basic::*;
use bitsplain_core::nom::combinator::value;
use bitsplain_core::nom::number::complete::*;
use bitsplain_core::parse::*;
use bitsplain_core::value::Value;
use bitsplain_core::*;

use bitsplain_core::datatypes::*;

pub fn payment_code(s: Span) -> IResult<Span, ()> {
    let (s, _) = p(
        value(0x47, u8),
        ann(
            "Prefix",
            Value::Alt(
                Box::new(Value::Bytes(vec![0x47])),
                Box::new(Value::String("P".to_string())),
            ),
        ),
    )(s)?;
    let (s, _version) = p(u8, ann("Version", auto()))(s)?;
    let (s, _flags) = p(u8, ann("Feature bit field", auto()))(s)?;
    // let (s, sig_required) = anv("Signature required", u8)(s)?;
    let (s, _x) = p(public_key, ann("X value", auto()))(s)?;
    let (s, _chain) = p(chain_hash_le, ann("Chain code", auto()))(s)?;
    let (s, _reserved) = p(
        bytes(13usize),
        ann("Reserved for future", |b: &Vec<_>| Value::Bytes(b.clone())),
    )(s)?;

    Ok((s, ()))
}
