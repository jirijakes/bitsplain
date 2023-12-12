use crate::bitcoin::ScriptBuf;
use crate::dsl::*;
use crate::parse::*;
use crate::types::*;
use crate::value::Value;

pub fn script(input: Span) -> Parsed<ScriptBuf> {
    let (s, len) = parse(varint, ann("Length", auto()))(input)?;
    let (s, x) = parse(
        bytes(len),
        ann("Script Data", |b: &Vec<_>| Value::bytes(b.to_vec())),
    )(s)?;
    let script: ScriptBuf = x.to_vec().into();
    if !script.is_empty() {
        s.insert(ann("Script", Value::Script(script.clone())));
    }
    Ok((s.with("datatype", "script"), script))
}
