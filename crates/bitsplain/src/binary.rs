//! Functions decoding textual input according to various encoding schemes.

use std::ops::Deref;

use bech32::primitives::decode::CheckedHrpstring;
use bech32::NoChecksum;
use bytes::Bytes;

/// Binary data with information about their origin.
#[derive(Clone, Debug)]
pub enum Binary {
    Hex(Bytes),
    Base58Check(Bytes),
    Base64(Bytes),
    Bech32(String, Bytes),
    Raw(Bytes),
}

impl Deref for Binary {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        match self {
            Binary::Hex(v) => v,
            Binary::Base58Check(v) => v,
            Binary::Base64(v) => v,
            Binary::Raw(v) => v,
            Binary::Bech32(_, v) => v,
        }
    }
}

/// Attempt to decode string as hexadecimal string.
pub fn string_to_hex(s: &str) -> Option<Binary> {
    hex::decode(s).ok().map(|b| Binary::Hex(b.into()))
}

/// Attempt to decode string as Base64-encoded string.
pub fn string_to_base64(s: &str) -> Option<Binary> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(s)
        .ok()
        .map(|b| Binary::Base64(b.into()))
}

/// Attempt to decode string as Base58-encoded string.
pub fn string_to_base58(s: &str) -> Option<Binary> {
    use bitcoin::base58::*;
    decode_check(s).ok().map(|b| Binary::Base58Check(b.into()))
}

/// Attempt to decode string as Bech32-encoded string without checksum.
pub fn string_to_bech32(s: &str) -> Option<Binary> {
    CheckedHrpstring::new::<NoChecksum>(s)
        .ok()
        .map(|ch| Binary::Bech32(ch.hrp().to_string(), ch.byte_iter().collect()))
}

/// Attempt to decode raw byets as string.
pub fn binary_to_string(b: &[u8]) -> Option<String> {
    String::from_utf8(b.to_vec()).ok()
}
