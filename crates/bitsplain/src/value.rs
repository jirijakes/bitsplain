//! `Value` is intermediate type between domain-specific types and final presentation.

use std::fmt::Display;

// use bitcoin::hashes::hex::ToHex;
use bitcoin::hashes::{sha256d, Hash};
use bitcoin::secp256k1::ecdsa::Signature;
use bitcoin::{Address, BlockHash, PublicKey, ScriptBuf, Txid};
use bytes::Bytes;
use time::OffsetDateTime;

use crate::types::Sat;

/// Set of primitive values that can be formatted depending on the context.
/// Parsing any binary data will result in a [`Tree`](crate::tree::Tree) of these values.
#[derive(Clone, Debug)]
pub enum Value {
    /// Bitcoin address.
    Addr(Option<Address>),

    /// Any integral value.
    Num(i128),

    /// Byte size.
    Size(u64),

    /// Any arbitrary byte array.
    Bytes(Bytes),

    /// Bitcoin script.
    Script(ScriptBuf),

    /// ECDSA signature.
    Signature(Signature),

    /// ECDSA public key
    PublicKey(PublicKey),

    /// Formatted text.
    Text {
        text: String,
        foreground: Option<[u8; 3]>,
        background: Option<[u8; 3]>,
    },

    /// Hash (block hash, txid etc.).
    Hash(sha256d::Hash),

    /// Any sort of timestamp.x
    Timestamp(time::OffsetDateTime),

    /// Alternative values.
    Alt(Box<Value>, Box<Value>),

    /// Bitcoin amount in satoshis.
    Sat(Sat),

    /// No value.
    Nil,
}

impl Value {
    /// Creates alternative from two distinct values.
    #[inline]
    pub fn alt(v1: Value, v2: Value) -> Value {
        Value::Alt(Box::new(v1), Box::new(v2))
    }

    /// Creates value from bytes.
    #[inline]
    pub fn bytes<I: Into<Bytes>>(bytes: I) -> Value {
        Value::Bytes(bytes.into())
    }

    /// Creates value from a number.
    #[inline]
    pub fn num(num: impl Into<i128>) -> Value {
        Value::Num(num.into())
    }

    /// Creates textual value from anything that has [`Display`].
    #[inline]
    pub fn display<S: Display>(value: S) -> Value {
        Self::text(value.to_string())
    }

    /// Creates formatted textual value from anything that has [`Display`].
    #[inline]
    pub fn display_fmt<S: Display>(
        value: S,
        foreground: Option<[u8; 3]>,
        background: Option<[u8; 3]>,
    ) -> Value {
        Self::text_fmt(value.to_string(), foreground, background)
    }

    /// Creates textual value.
    #[inline]
    pub fn text<S: AsRef<str>>(text: S) -> Value {
        Value::Text {
            text: text.as_ref().to_string(),
            foreground: None,
            background: None,
        }
    }

    /// Creates formatted textual value.
    #[inline]
    pub fn text_fmt<S: AsRef<str>>(
        text: S,
        foreground: Option<[u8; 3]>,
        background: Option<[u8; 3]>,
    ) -> Value {
        Value::Text {
            text: text.as_ref().to_string(),
            foreground,
            background,
        }
    }
}

impl Value {
    pub fn preview(&self) -> String {
        match self {
            Value::Addr(a) => format!("{:?}", a),
            Value::Num(n) => n.to_string(),
            Value::Size(s) => s.to_string(),
            Value::Bytes(b) => hex::encode(b),
            Value::Script(s) => s.to_string(),
            Value::Signature(s) => s.to_string(), //.serialize_compact().to_hex(),
            Value::PublicKey(k) => k.to_string(),
            Value::Text { text, .. } => text.to_string(),
            Value::Hash(id) => id.to_string(),
            Value::Alt(v1, v2) => format!("{}/{}", v1.preview(), v2.preview()),
            Value::Sat(s) => s.as_str(),
            Value::Nil => "".to_string(),
            Value::Timestamp(ts) => ts.to_string(),
        }
    }
}

pub trait ToValue {
    fn to_value(&self) -> Value;
}

impl ToValue for () {
    fn to_value(&self) -> Value {
        Value::Nil
    }
}

impl ToValue for i32 {
    fn to_value(&self) -> Value {
        Value::Num(*self as i128)
    }
}

impl ToValue for u16 {
    fn to_value(&self) -> Value {
        Value::Num(*self as i128)
    }
}

impl ToValue for u8 {
    fn to_value(&self) -> Value {
        Value::Num(*self as i128)
    }
}

impl ToValue for Sat {
    fn to_value(&self) -> Value {
        Value::Sat(*self)
    }
}

impl ToValue for Signature {
    fn to_value(&self) -> Value {
        Value::Signature(*self)
    }
}

impl ToValue for PublicKey {
    fn to_value(&self) -> Value {
        Value::PublicKey(*self)
    }
}

impl ToValue for BlockHash {
    fn to_value(&self) -> Value {
        Value::bytes(self.to_raw_hash().to_byte_array().to_vec())
    }
}

impl ToValue for u32 {
    fn to_value(&self) -> Value {
        Value::Num(*self as i128)
    }
}

impl ToValue for u64 {
    fn to_value(&self) -> Value {
        Value::Num(*self as i128)
    }
}

impl ToValue for &[u8] {
    fn to_value(&self) -> Value {
        Value::bytes(self.to_vec())
    }
}

impl ToValue for Vec<u8> {
    fn to_value(&self) -> Value {
        Value::bytes(self.clone())
    }
}

impl ToValue for &str {
    fn to_value(&self) -> Value {
        Value::text(self)
    }
}

impl ToValue for bool {
    fn to_value(&self) -> Value {
        if *self {
            Value::Num(1.into())
        } else {
            Value::Num(0.into())
        }
    }
}

impl ToValue for sha256d::Hash {
    fn to_value(&self) -> Value {
        Value::Hash(*self)
    }
}

impl ToValue for OffsetDateTime {
    fn to_value(&self) -> Value {
        Value::Alt(
            Box::new(Value::Num(self.unix_timestamp().into())),
            Box::new(Value::Timestamp(*self)),
        )
    }
}

impl ToValue for Txid {
    fn to_value(&self) -> Value {
        Value::Hash(self.to_raw_hash())
    }
}

impl ToValue for ScriptBuf {
    fn to_value(&self) -> Value {
        Value::Script(self.clone())
    }
}

impl<A: ToValue, B: ToValue> ToValue for (A, B) {
    fn to_value(&self) -> Value {
        Value::alt(self.0.to_value(), self.1.to_value())
    }
}
