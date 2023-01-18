use crate::ann::{ann, auto};
use crate::nom::combinator::value;
use crate::nom::number::complete::*;
use crate::parse::*;
use crate::types::*;
use crate::value::Value;
use crate::*;

pub fn payment_code(s: Span) -> IResult<Span, ()> {
    let (s, _) = p(
        value(0x47, u8),
        ann(
            "Prefix",
            Value::Alt(
                Box::new(Value::bytes(vec![0x47])),
                Box::new(Value::text("P")),
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
        ann("Reserved for future", |b: &Vec<_>| Value::bytes(b.clone())),
    )(s)?;

    Ok((s, ()))
}
