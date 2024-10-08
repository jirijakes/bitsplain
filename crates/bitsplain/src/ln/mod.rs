use nom::combinator::{map, success};
use nom::number::complete::{be_u32, be_u64};

use crate::dsl::{ann, auto};
use crate::nom::number::complete::{be_u16, be_u24, u8};
use crate::parse::*;
use crate::value::{ToValue, Value};
use crate::*;

pub mod bolt12;
pub mod gossip;

pub fn bigsize(s: Span) -> Parsed<u64> {
    let (s, first) = u8(s)?;
    match first {
        0xFF => be_u64(s),
        0xFE => map(be_u32, |n| n as u64)(s),
        0xFD => map(be_u16, |n| n as u64)(s),
        n => success(n as u64)(s),
    }
}

/// Internal representation of short channel ID (SCID). Crate `lightning` normally
/// uses `u64` representation to which `ShortChannelId` can be converted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShortChannelId {
    pub block: u32,
    pub tx: u32,
    pub output: u16,
}

impl ShortChannelId {
    /// Textual representation of short channel ID.
    pub fn as_string(&self) -> String {
        format!("{}x{}x{}", self.block, self.tx, self.output)
    }
}

impl From<ShortChannelId> for u64 {
    fn from(sci: ShortChannelId) -> Self {
        let b = sci.block.to_be_bytes()[0..3].to_vec();
        let t = sci.tx.to_be_bytes()[0..3].to_vec();
        let o = sci.output.to_be_bytes().to_vec();

        u64::from_be_bytes(vec![b, t, o].concat().try_into().unwrap())
    }
}

impl ToValue for ShortChannelId {
    fn to_value(&self) -> value::Value {
        Value::text(self.as_string())
    }
}

/// Parser of short channel ID (SCID). Reads 8 bytes.
pub fn short_channel_id(s: Span) -> Parsed<ShortChannelId> {
    let (s, block) = parse(be_u24, ann("Block height", auto()))(s)?;
    let (s, tx) = parse(be_u24, ann("Transaction index", auto()))(s)?;
    let (s, output) = parse(be_u16, ann("Output index", auto()))(s)?;

    Ok((s, ShortChannelId { block, tx, output }))
}

pub struct RgbColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl ToValue for RgbColor {
    fn to_value(&self) -> Value {
        Value::Alt(
            Box::new(Value::text(format!(
                "#{:02x}{:02x}{:02x}",
                self.red, self.green, self.blue
            ))),
            Box::new(Value::text_fmt(
                "  ",
                None,
                Some([self.red, self.green, self.blue]),
            )),
        )
    }
}

pub fn rgb_color(s: Span) -> Parsed<RgbColor> {
    let (s, red) = parse(u8, ann("Red", auto()))(s)?;
    let (s, green) = parse(u8, ann("Green", auto()))(s)?;
    let (s, blue) = parse(u8, ann("Blue", auto()))(s)?;

    Ok((s, RgbColor { red, green, blue }))
}
