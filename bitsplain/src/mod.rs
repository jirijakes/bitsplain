#![feature(never_type)]

pub mod ann;
pub mod annotations;
pub mod basic;
pub mod binary;
pub mod datatypes;
pub mod decode;
pub mod lines;
pub mod parse;
pub mod tree;
pub mod value;

use annotations::Annotations;
use binary::Binary;
use decode::Decoder;
use parse::Annotated;
use time::OffsetDateTime;

pub type Span<'a> = Annotated<&'a [u8]>;

/// Input from user.
#[derive(Clone, Debug)]
pub enum Input {
    /// User provided directly string (via argument).
    String(String),

    /// User provided some binary data (via stdin or file).
    /// The data could be interpreted either as raw or as string.
    Binary(Vec<u8>),
}

#[derive(Debug)]
pub struct Candidate {
    pub decoder: &'static Decoder,
    pub annotations: Annotations,
    pub data: Binary,
}

pub fn format_time(time: &OffsetDateTime) -> String {
    time.format(
        &time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap(),
    )
    .unwrap()
}
