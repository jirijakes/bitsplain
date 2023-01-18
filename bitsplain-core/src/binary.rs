//! Functions decoding textual input according to various encoding schemes.

/// Binary data with information about their origin.
#[derive(Clone, Debug)]
pub enum Binary {
    Hex(Vec<u8>),
    Base58Check(Vec<u8>),
    Base64(Vec<u8>),
    Bech32(String, Vec<u8>),
    Raw(Vec<u8>),
}

impl Binary {
    pub fn bytes(&self) -> &[u8] {
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
    hex::decode(s).ok().map(Binary::Hex)
}

/// Attempt to decode string as Base64-encoded string.
pub fn string_to_base64(s: &str) -> Option<Binary> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(s)
        .ok()
        .map(Binary::Base64)
}

/// Attempt to decode string as Base58-encoded string.
pub fn string_to_base58(s: &str) -> Option<Binary> {
    use bitcoin::util::base58::*;
    from_check(s).ok().map(Binary::Base58Check)
}

/// Attempt to decode string as Bech32-encoded string without checksum.
pub fn string_to_bech32(s: &str) -> Option<Binary> {
    bech32::decode_without_checksum(s)
        .ok()
        .and_then(|(hrp, data)| {
            bech32::convert_bits(&data, 5, 8, false)
                .ok()
                .map(|data| Binary::Bech32(hrp, data))
        })
}

/// Attempt to decode raw byets as string.
pub fn binary_to_string(b: &[u8]) -> Option<String> {
    String::from_utf8(b.to_vec()).ok()
}
