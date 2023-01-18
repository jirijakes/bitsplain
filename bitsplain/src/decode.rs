//! Core types and functions related to decoding of binary data.

use crate::annotations::Annotations;
use crate::binary::*;
use crate::{Candidate, Input};

/// Description of a function that can decode data. During program's
/// execution, decoders are tried one by one and those that return a result
/// are collected.
pub struct Decoder {
    /// Name of data returned by this decoder.
    pub title: &'static str,

    pub group: &'static str,

    pub symbol: &'static str,

    /// Decoding function.
    pub decode: fn(&Binary) -> Option<Annotations>,
}

impl std::fmt::Debug for Decoder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Decoder")
            .field("name", &self.title)
            .finish()
    }
}

// So instances of Decoder struct can be registered and used.
inventory::collect!(Decoder);

/// List of all known decoders.
pub fn all_decoders() -> Vec<&'static Decoder> {
    inventory::iter::<Decoder>().collect()
}

/// Attempt to decode input with the best effort.
/// Zero, one or more results can be returned.
pub fn decode_input(input: Input) -> Vec<Candidate> {
    input_to_binaries(input)
        .iter()
        .flat_map(|b| {
            all_decoders().into_iter().map(|d| {
                (d.decode)(b).map(|a| Candidate {
                    decoder: d,
                    annotations: a,
                    data: b.clone(),
                })
            })
        })
        .flatten()
        .collect()
}

/// From input extract all possible interpreations of binary data.
/// The input can be interpreted either as raw binary data or as
/// a string representing some known encoding of binary data.
/// The results are ordered from more likely to less likely, i. e.
/// since Base16 may also be a valid Base64, the decoded Base16
/// will preceed the other.
fn input_to_binaries(input: Input) -> Vec<Binary> {
    match input {
        Input::String(s) => try_decode_string(&s),
        Input::Binary(b) => {
            let mut s = binary_to_string(&b)
                .map(|s| try_decode_string(&s))
                .unwrap_or_default();

            // Let's put raw bytes to the end. If raw bytes
            // were indeed provided in the input, most likely
            // all the attempts to decode them as string would
            // have failed and only the raw bytes will remain.
            s.push(Some(Binary::Raw(b)));

            s
        }
    }
    .into_iter()
    .flatten()
    .collect()
}

/// Attempt to decode given string as binary data according
/// to various encoding schemes.
#[inline]
fn try_decode_string(s: &str) -> Vec<Option<Binary>> {
    vec![
        string_to_hex(s),
        string_to_bech32(s),
        string_to_base58(s),
        string_to_base64(s),
    ]
}
