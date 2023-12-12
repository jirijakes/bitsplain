//! Domain-specific language for defining annotations.

use std::marker::PhantomData;

use crate::tree::Tag;
use crate::value::{ToValue, Value};

/// Represents a function that can create a [`Value`] out of `T`.
///
/// Its main purpose is to mark all inputs for function [`ann`] and
/// method [`Ann::value`] that can be used as generators for value.
/// These can be, for example, a static string or directly provided
/// [`Value`].
pub enum Make<T, V> {
    Fn(Box<dyn Fn(&T) -> V>),
    Static(V),
    Empty,
}

impl<T> Make<T, Value> {
    /// Make a value out of input.
    pub fn resolve(&self, input: &T) -> Value {
        match self {
            Make::Fn(f) => (f)(input),
            Make::Static(v) => v.clone(),
            Make::Empty => Value::Nil,
        }
    }

    /// Make a value if and only if it does not depend on any input.
    pub fn resolve_static(&self) -> Option<Value> {
        match self {
            Make::Static(v) => Some(v.clone()),
            _ => None,
        }
    }
}

impl<T> Make<T, Tag> {
    pub fn resolve(&self, input: &T) -> Option<Tag> {
        match self {
            Make::Fn(f) => Some((f)(input)),
            Make::Static(t) => Some(t.clone()),
            Make::Empty => None,
        }
    }
}

impl<T> Make<T, String> {
    /// Make a value out of input.
    pub fn resolve(&self, input: &T) -> Option<String> {
        match self {
            Make::Fn(f) => Some((f)(input)),
            Make::Static(v) => Some(v.clone()),
            Make::Empty => None,
        }
    }

    /// Make a value if and only if it does not depend on any input.
    pub fn resolve_static(&self) -> Option<String> {
        match self {
            Make::Static(v) => Some(v.clone()),
            _ => None,
        }
    }
}

impl<T> From<&'static str> for Make<T, Value> {
    fn from(s: &'static str) -> Self {
        Make::Static(Value::text(s))
    }
}

impl<T> From<Value> for Make<T, Value> {
    fn from(value: Value) -> Self {
        Make::Static(value)
    }
}

impl<T: ToValue> From<Auto<T>> for Make<T, Value> {
    fn from(_: Auto<T>) -> Self {
        Make::Fn(Box::new(|t: &T| t.to_value()))
    }
}

impl<T, F, O> From<F> for Make<T, O>
where
    F: Fn(&T) -> O + 'static,
{
    fn from(f: F) -> Self {
        Make::Fn(Box::new(f))
    }
}

impl<T> From<&'static str> for Make<T, String> {
    fn from(s: &'static str) -> Self {
        Make::Static(s.to_string())
    }
}

impl<T> From<String> for Make<T, String> {
    fn from(s: String) -> Self {
        Make::Static(s)
    }
}

#[derive(Clone, Debug)]
/// External reference attached to an annotation.
pub enum Reference {
    /// Reference to a web page.
    Www(String),
    /// Reference to a BIP.
    Bip(u16),
    // Code,
}

/// Collection of various annotations of a parsed field.
pub struct Ann<T> {
    /// Label of the field.
    pub label: String,
    /// Interpreted value of the content of the field.
    pub value: Make<T, Value>,
    /// Documentation string.
    pub doc: Option<String>,
    /// External references.
    pub refs: Vec<Reference>,
    /// Tags of the fields.
    pub tags: Vec<Make<T, Tag>>,
    /// Splain string.
    pub splain: Make<T, String>,
}

impl<T> Ann<T> {
    /// Add splain.
    pub fn splain(mut self, s: impl Into<Make<T, String>>) -> Ann<T> {
        self.splain = s.into();
        self
    }

    /// Add documentation.
    pub fn doc(mut self, s: impl AsRef<str>) -> Ann<T> {
        self.doc = Some(s.as_ref().to_string());
        self
    }

    /// Add reference to a web page; may be called repeatedly.
    pub fn www(mut self, s: impl AsRef<str>) -> Ann<T> {
        self.refs.push(Reference::Www(s.as_ref().into()));
        self
    }

    /// Add reference to a BIP; may be called repeatedly.
    pub fn bip(mut self, bipno: u16) -> Ann<T> {
        self.refs.push(Reference::Bip(bipno));
        self
    }

    /// Set label.
    pub fn label(mut self, s: impl AsRef<str>) -> Ann<T> {
        self.label = s.as_ref().to_string();
        self
    }

    /// Add tag.
    pub fn tag(mut self, s: impl Into<Make<T, Tag>>) -> Ann<T> {
        self.tags.push(s.into());
        self
    }

    /// Set interpreted value of the content.
    pub fn value(mut self, e: impl Into<Make<T, Value>>) -> Ann<T> {
        self.value = e.into();
        self
    }
}

/// Marker for [`Make`] that creates value out of [`ToValue`].
pub struct Auto<T: ToValue>(PhantomData<T>);

/// Automatically derives [`Value`] from [`ToValue`] instance.
pub const fn auto<T: ToValue>() -> Auto<T> {
    Auto(PhantomData)
}

/// Creates a new annotation with `label` and a value generator. All optional fields
/// are empty and can be later popupated by calling appropriate method on [`Ann`].
pub fn ann<T>(label: impl AsRef<str>, value: impl Into<Make<T, Value>>) -> Ann<T> {
    Ann {
        label: label.as_ref().to_string(),
        value: value.into(),
        tags: vec![],
        refs: vec![],
        doc: None,
        splain: Make::Empty,
    }
}
