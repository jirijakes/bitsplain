//! Bitsplain is a library that helps people understand Bitcoin-related
//! binary data. When provided with some data — be it a binary file, hex-encoded
//! string or any other commonly used encoding scheme — Bitsplain first tries to
//! identify what the data represent and if it succeeds it offers an explanation
//! of the data through “annotations”. These annotations consist of description, data type,
//! rendered value, position in the binary input etc.
//!
//! The library does not interpret the annotations, however crate `bitsplain-bin`
//! offers two user interfaces, a CLI and GTK.
//!
//! ## Use it
//!
//! Calling `bitsplain::decode::decode_input(input)` will return a vector of [`candidates`](crate::decode::Candidate).
//! Each of the candidates contains reference to [`decoder`](crate::decode::Decoder) which successfully
//! parsed the data, the [`annotations`](crate::tree::Annotations) and view over original binary data.
//!
//! ## How it works?
//!
//! Bitsplain uses [`nom`] to implement parsers of all the supported data formats.
//! Using custom parser implementation it can track every value that the parser
//! returns and its position within the data. All this information is assembled
//! in a tree of [`Values`](crate::value), which is then returned for interpretation. Writers of data parsers
//! can use a convenient [`DSL`](dsl).
//!
pub use {bitcoin, hex, nom};

pub mod binary;
pub mod decode;
pub mod dsl;
pub mod parse;
pub mod tree;
pub mod types;
pub mod value;

mod btc;
mod ln;

/// Registers new decoder, defined by parser function, under a specified name.
/// Optionally a condidition, in form of a pattern match, can be added.
#[rustfmt::skip]
#[macro_export]
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
            $crate::decode::Decoder {
                title: $title,
                group: $group,
                symbol: $symbol,
                decode: |b| {
                    if matches!(b, $( $pattern )|+ $( if $guard )?) {
                        $func($crate::parse::Annotated::new(&b)).ok().and_then(|(x, _)| {
                            use $crate::nom::InputLength;
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
    crate::btc::block::block_header,
    b if b.len() == 80
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
    crate::binary::Binary::Bech32(hrp, _ ) if hrp == "lno",
);

decoder!(
    title = "Lightning Network BOLT 12 invoice request",
    group = "ln",
    symbol = "bolt12r",
    crate::ln::bolt12::bolt12,
    crate::binary::Binary::Bech32(hrp, _ ) if hrp == "lnr",
);

decoder!(
    title = "Lightning Network BOLT 12 invoice",
    group = "ln",
    symbol = "bolt12i",
    crate::ln::bolt12::bolt12,
    crate::binary::Binary::Bech32(hrp, _ ) if hrp == "lni",
);

decoder!(
    title = "BIP-47 payment code",
    group = "btc",
    symbol = "bip47",
    crate::btc::bip47::payment_code,
    crate::binary::Binary::Base58Check(b) if b.first() == Some(&0x47)
);

decoder!(
    title = "Bitcoin transaction prevout",
    group = "btc",
    symbol = "prevout",
    crate::btc::tx::tx_out
);
