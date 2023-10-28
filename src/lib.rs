#![cfg_attr(not(feature = "std"), no_std)]

pub mod de;
pub mod error;
#[cfg(feature = "std")]
pub mod ser;

pub use error::{Error, Result};

/// Represents all possible KeyValues values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Value {
    String(String),
    Object(Object),
}

/// Represents a KeyValues object.
#[cfg(feature = "preserve_order")]
pub type Object = indexmap::IndexMap<String, Vec<Value>>;

/// Represents a KeyValues object.
#[cfg(not(feature = "preserve_order"))]
pub type Object = std::collections::BTreeMap<String, Vec<Value>>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RootKind {
    Nested(String),
    Flattened,
}

/// Indicates that a type can be represented as a KeyValues document.
pub trait KeyValuesRoot {
    fn kind() -> RootKind;
}

/// Represents a KeyValues document.
///
/// Note: A document typically consists of a single key-object pair. However, this library
/// allows multiple root keys to exist simultaneously. This is because some implementations
/// of KeyValues (such as the VMF format) *do* permit multiple root keys.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyValues {
    pub root: Object,
}

impl KeyValues {
    /// Creates a KeyValues from a single key-value pair.
    pub fn new(key: String, value: Value) -> Self {
        let mut root = Object::new();
        root.insert(key, vec![value]);
        Self { root }
    }

    /// Creates a KeyValues from a root object that may contain multiple keys.
    pub fn with_root(root: Object) -> Self {
        Self { root }
    }
}

impl KeyValuesRoot for KeyValues {
    fn kind() -> RootKind {
        RootKind::Flattened
    }
}

#[cfg(feature = "std")]
impl serde::Serialize for KeyValues {
    fn serialize<S>(&self, _serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!()
    }
}

impl<'de> serde::Deserialize<'de> for KeyValues {
    fn deserialize<D>(_deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!()
    }
}
