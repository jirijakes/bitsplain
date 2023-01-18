use bitsplain_core::ann::*;
use bitsplain_core::basic::*;
use bitsplain_core::bitcoin::Script;
use bitsplain_core::nom::IResult;
use bitsplain_core::parse::*;
use bitsplain_core::value::Value;
use bitsplain_core::Span;

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
