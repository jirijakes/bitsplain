use annotations::Annotations;
use binary::Binary;
use decode::Decoder;
pub use nom::IResult;
use parse::Annotated;
use time::OffsetDateTime;
pub use {bitcoin, hex, nom};

pub mod ann;
pub mod annotations;
pub mod binary;
pub mod decode;
pub mod lines;
pub mod parse;
pub mod tree;
pub mod types;
pub mod value;

mod btc;
mod ln;

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

pub enum Void {}

/// Registers new decoder, defined by parser function, under a specified name.
/// Optionally a condidition, in form of a pattern match, can be added.
#[rustfmt::skip]
macro_rules! decoder {
    (
        title = $title: literal,
        group = $group: literal,
        symbol = $symbol: literal,
        $func: path $(,)?) => {
        decoder!(title = $title, group = $group, symbol = $symbol, $func, _);
    };
    (
        title = $title: literal,
        group = $group: literal,
        symbol = $symbol: literal,
        $func: path,
        $(|)? $( $pattern:pat_param )|+ $( if $guard: expr )? $(,)?) => {
        inventory::submit! {
            crate::decode::Decoder {
                title: $title,
                group: $group,
                symbol: $symbol,
                decode: |b| {
                    if matches!(b, $( $pattern )|+ $( if $guard )?) {
                        $func(crate::parse::Annotated::new(b.bytes())).ok().and_then(|(x, _)| {
                            use crate::nom::InputLength;
                            if x.input_len() > 0 {
                                None
                            } else {
                                Some(x.annotations())
                            }
                        })
                    } else {
                        None
                    }
                }
            }
        }
    };
}

decoder!(
    title = "Bitcoin block header",
    group = "btc",
    symbol = "header",
    crate::btc::block::block_header
);

decoder!(
    title = "Serialized Bitcoin transaction",
    group = "btc",
    symbol = "tx",
    crate::btc::tx::tx
);

decoder!(
    title = "Bitcoin script",
    group = "btc",
    symbol = "script",
    crate::btc::datatypes::script
); // without script_len1

decoder!(
    title = "Lightning Network channel announcement",
    group = "ln",
    symbol = "chan_ann",
    crate::ln::gossip::channel_announcement
);

decoder!(
    title = "Lightning Network channel update",
    group = "ln",
    symbol = "chan_upd",
    crate::ln::gossip::channel_update
);

decoder!(
    title = "Lightning Network node announcement",
    group = "ln",
    symbol = "node_ann",
    crate::ln::gossip::node_announcement
);

decoder!(
    title = "Lightning Network gossip timestamp filter",
    group = "ln",
    symbol = "ts_filter",
    crate::ln::gossip::gossip_timestamp_filter
);

decoder!(
    title = "Lightning Network BOLT 12 offer",
    group = "ln",
    symbol = "bolt12o",
    crate::ln::bolt12::bolt12,
    Binary::Bech32(hrp, _ ) if hrp == "lno",
);

decoder!(
    title = "Lightning Network BOLT 12 invoice request",
    group = "ln",
    symbol = "bolt12r",
    crate::ln::bolt12::bolt12,
    Binary::Bech32(hrp, _ ) if hrp == "lnr",
);

decoder!(
    title = "Lightning Network BOLT 12 invoice",
    group = "ln",
    symbol = "bolt12i",
    crate::ln::bolt12::bolt12,
    Binary::Bech32(hrp, _ ) if hrp == "lni",
);

decoder!(
    title = "BIP-47 payment code",
    group = "btc",
    symbol = "bip47",
    crate::btc::bip47::payment_code,
    Binary::Base58Check(b) if b.first() == Some(&0x47)
);

decoder!(
    title = "Bitcoin transaction prevout",
    group = "btc",
    symbol = "prevout",
    crate::btc::tx::tx_out
);
