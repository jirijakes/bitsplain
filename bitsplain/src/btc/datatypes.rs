use crate::ann::*;
use crate::bitcoin::Script;
use crate::types::*;
use crate::nom::IResult;
use crate::parse::*;
use crate::value::Value;
use crate::Span;

pub fn script(input: Span) -> IResult<Span, Script> {
    let (s, len) = p(varint, ann("script_len", auto()))(input)?;
    let (s, x) = p(
        bytes(len),
        ann("script data", |b: &Vec<_>| Value::Bytes(b.to_vec())),
    )(s)?;
    let script: Script = x.to_vec().into();
    s.insert(ann("script", Value::Script(script.clone())));
    Ok((s.with("datatype", "script"), script))
}
