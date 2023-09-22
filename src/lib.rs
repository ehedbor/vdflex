#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "serde")]
pub mod error;
#[cfg(all(feature = "serde", feature = "std"))]
pub mod ser;
#[cfg(feature = "serde")]
pub mod de;

#[cfg(feature = "serde")]
pub use error::{Error, Result};

/// Represents a Keyvalues key.
pub type Key = String;

/// Represents all possible Keyvalues values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Value {
    String(String),
    Object(Object),
}

/// Represents a Keyvalues object.
#[cfg(feature = "preserve_order")]
pub type Object = indexmap::IndexMap<Key, Vec<Value>>;

/// Represents a Keyvalues object.
#[cfg(not(feature = "preserve_order"))]
pub type Object = std::collections::BTreeMap<Key, Vec<Value>>;

/// Represents a Keyvalues document.
///
/// Note: A document typically consists of a single key-object pair. However, this library
/// allows multiple root keys to exist simultaneously. This is because some implementations
/// of Keyvalues (such as the VMF format) *do* permit multiple root keys.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Keyvalues {
    pub root: Object,
}

impl Keyvalues {
    /// Creates a Keyvalues from a single key-value pair.
    pub fn new(key: String, value: Value) -> Self {
        let mut root = Object::new();
        root.insert(key, vec![value]);
        Self { root }
    }

    /// Creates a Keyvalues from a root object that may contain multiple keys.
    pub fn with_root(root: Object) -> Self {
        Self { root }
    }
}
