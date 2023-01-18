use bitcoin::{
    hashes::{hex::ToHex, sha256d},
    secp256k1::ecdsa::Signature,
    Address, BlockHash, PublicKey, Script, Txid,
};
use time::OffsetDateTime;

use crate::types::Sat;

#[derive(Clone, Debug)]
pub enum Value {
    /// Bitcoin address.
    Addr(Option<Address>),

    /// Any integral value.
    Num(i128),

    /// Byte size.
    Size(u64),

    /// Any arbitrary byte array.
    Bytes(Vec<u8>),

    /// Bitcoin script.
    Script(Script),

    /// ECDSA signature.
    Signature(Signature),

    /// ECDSA public key
    PublicKey(PublicKey),

    /// Text.
    String(String),

    /// Formatted text.
    Text {
        text: String,
        foreground: Option<[u8; 3]>,
        background: Option<[u8; 3]>,
    },

    /// Hash (block hash, txid etc.).
    Hash(sha256d::Hash),

    Timestamp(time::OffsetDateTime),

    /// Alternative values.
    Alt(Box<Value>, Box<Value>),

    /// Bitcoin amount in satoshis.
    Sat(Sat),

    /// No value.
    Nil,
}

impl Value {
    pub fn preview(&self) -> String {
        match self {
            Value::Addr(a) => format!("{:?}", a),
            Value::Num(n) => n.to_string(),
            Value::Size(s) => s.to_string(),
            Value::Bytes(b) => hex::encode(b),
            Value::Script(s) => s.to_string(),
            Value::Signature(s) => s.serialize_compact().to_hex(),
            Value::PublicKey(k) => k.to_string(),
            Value::String(s) => s.to_string(),
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
        Value::Bytes(self.as_hash().to_vec())
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
        Value::Bytes(self.to_vec())
    }
}

impl ToValue for &str {
    fn to_value(&self) -> Value {
        Value::String(self.to_string())
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
        Value::Hash(self.as_hash())
    }
}

impl ToValue for Script {
    fn to_value(&self) -> Value {
        Value::Script(self.clone())
    }
}
