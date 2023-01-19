use crate::bitcoin::Script;
use crate::dsl::*;
use crate::parse::*;
use crate::types::*;
use crate::value::Value;

pub fn script(input: Span) -> Parsed<Script> {
    let (s, len) = parse(varint, ann("script_len", auto()))(input)?;
    let (s, x) = parse(
        bytes(len),
        ann("script data", |b: &Vec<_>| Value::bytes(b.to_vec())),
    )(s)?;
    let script: Script = x.to_vec().into();
    s.insert(ann("script", Value::Script(script.clone())));
    Ok((s.with("datatype", "script"), script))
}
