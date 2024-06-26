#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

mod de;
pub mod error;
#[cfg(feature = "std")]
pub mod ser;

pub use error::{Error, Result};
pub use ser::{
    kv_to_string, kv_to_string_pretty, kv_to_writer, kv_to_writer_pretty, to_string,
    to_string_pretty, to_writer, to_writer_pretty,
};

use std::fmt;
use std::result;

/// Represents all possible VDF values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Value {
    /// Stores a [`String`] value.
    String(String),
    /// Stores an [`Object`] value.
    Object(Object),
}

#[cfg(feature = "std")]
impl serde::Serialize for Value {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> result::Result<S::Ok, S::Error> {
        match self {
            Value::String(string) => string.serialize(serializer),
            Value::Object(object) => object.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for Value {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> result::Result<Self, D::Error> {
        deserializer.deserialize_any(ValueVisitor)
    }
}

struct ValueVisitor;

impl<'de> serde::de::Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a KeyValues string or object")
    }

    fn visit_str<E: serde::de::Error>(self, v: &str) -> result::Result<Self::Value, E> {
        Ok(Value::String(String::from(v)))
    }

    fn visit_string<E: serde::de::Error>(self, v: String) -> result::Result<Self::Value, E> {
        Ok(Value::String(v))
    }

    fn visit_map<A: serde::de::MapAccess<'de>>(
        self,
        mut map: A,
    ) -> result::Result<Self::Value, A::Error> {
        #[cfg(feature = "preserve_order")]
        use indexmap::map::Entry;
        #[cfg(not(feature = "preserve_order"))]
        use std::collections::btree_map::Entry;

        #[cfg(feature = "preserve_order")]
        let mut obj = Object::with_capacity(map.size_hint().unwrap_or(0));
        #[cfg(not(feature = "preserve_order"))]
        let mut obj = Object::new();

        while let Some((key, value)) = map.next_entry::<String, Value>()? {
            match obj.entry(key) {
                Entry::Occupied(mut oe) => {
                    oe.get_mut().push(value);
                }
                Entry::Vacant(ve) => {
                    ve.insert(vec![value]);
                }
            }
        }

        Ok(Value::Object(obj))
    }
}

/// Represents a KeyValues object.
#[cfg(feature = "preserve_order")]
pub type Object = indexmap::IndexMap<String, Vec<Value>>;

/// Represents a KeyValues object.
#[cfg(not(feature = "preserve_order"))]
pub type Object = std::collections::BTreeMap<String, Vec<Value>>;

/// Represents a KeyValues document.
///
/// Note: A document typically consists of a single key-object pair. However, this library
/// allows multiple root keys to exist simultaneously. This is because some implementations
/// of KeyValues (such as the VMF format) *do* permit multiple root keys.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyValues {
    /// The root object of the document.
    pub root: Object,
}

impl KeyValues {
    /// Creates a KeyValues from a single key-value pair. This is the typical way to create
    /// KeyValues document.
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

#[cfg(feature = "std")]
impl serde::Serialize for KeyValues {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> result::Result<S::Ok, S::Error> {
        self.root.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for KeyValues {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> result::Result<Self, D::Error> {
        Ok(Self::with_root(Object::deserialize(deserializer)?))
    }
}
