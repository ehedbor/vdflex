//! # VDFLex
//!
//! VDFLex is a (de)serialization library for parsing the Valve Data File format with
//! [serde](https://crates.io/crates/serde). VDF—or more generally, [KeyValues](https://developer.valvesoftware.com/wiki/KeyValues)—is
//! a data format developed by Valve for use in Steam and the Source engine.
//!
//! ```text
//! LightmappedGeneric
//! {
//!     $basetexture "myassets\gravel01"
//!     $surfaceprop gravel
//! }
//! ```
//!
//! ## Quick Start
//!
//! ```rust
//! use std::collections::BTreeMap;
//! use std::hash::Hash;
//! use serde::Serialize;
//!
//! #[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize)]
//! struct AppId(u32);
//!
//! #[derive(Serialize)]
//! #[serde(rename_all = "PascalCase")]
//! struct AppBuild {
//!     #[serde(rename = "AppID")]
//!     app_id: AppId,
//!     desc: String,
//!     content_root: String,
//!     build_output: String,
//!     depots: BTreeMap<AppId, Depot>
//! }
//!
//! #[derive(Serialize)]
//! struct Depot {
//!     #[serde(rename = "FileMapping")]
//!     file_mappings: Vec<FileMapping>
//! }
//!
//! #[derive(Serialize)]
//! struct FileMapping {
//!     #[serde(rename = "LocalPath")]
//!     local_path: String,
//!     #[serde(rename = "DepotPath")]
//!     depot_path: String,
//! }
//!
//! fn main() -> vdflex::Result<()> {
//!     let mut depots = BTreeMap::new();
//!     depots.insert(AppId(1234), Depot {
//!         file_mappings: vec![FileMapping {
//!             local_path: String::from("*"),
//!             depot_path: String::from("."),
//!         }],
//!     });
//!     
//!     let build_script = AppBuild {
//!         app_id: AppId(1234),
//!         desc: String::from("My SteamPipe build script"),
//!         content_root: String::from("..\\assets\\"),
//!         build_output: String::from("..\\build\\"),
//!         depots,
//!     };
//!     
//!     let text: String = vdflex::kv_to_string("AppBuild", &build_script)?;
//!     println!("{text}");
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Supported Types
//!
//! KeyValues is woefully underspecified, but in general it only supports strings and multimaps (objects). VDFLex attempts
//! to support every Rust type, but not all types necessarily have an "idiomatic" or "useful" representation. These are
//! the types that VDFlex supports and how they are represented in KeyValues:
//!
//!
//! |              Type              | Notes                                                                                                  |
//! |:------------------------------:|:-------------------------------------------------------------------------------------------------------|
//! |             `bool`             | Serialized to `1` or `0`                                                                               |
//! |            integers            | KeyValues doesn't typically support `i128` or `u128`                                                   |
//! |          `f32`/`f64`           | Some implementations only support `f32`. Non-finite floats are also poorly supported.                  |
//! |     `char`/`String`/`str`      | -                                                                                                      |
//! |            `Option`            | KeyValues has no equivalent of `null`, so `Some<T>` is represented as `T` and `None` is simply omitted |
//! |       Unit/Unit Structs        | Serialized like `None`                                                                                 |
//! |         Unit Variants          | Represented as a string matching the name of the variant                                               |
//! |        Newtype Structs         | Represented as the wrapped type                                                                        |
//! |        Newtype Variants        | Represented as an object mapping the variant name to the wrapped type                                  |
//! | Sequences/Tuples/Tuple Structs | Represented by repeating the key for each element in the sequence                                      |
//! |         Tuple Variants         | Represented by a map containing a sequence of the tuple's fields, using the variant name as the key    |
//! |          Maps/Structs          | Represented by objects (a curly bracket-enclosed list of key-value pairs)                              |
//! |        Struct Variants         | Represented as an object mapping the variant name to the struct representation of its fields           |
//!
//! ### Limitations
//!
//! - The *Bytes* type is unsupported, as there is no clear way to represent binary data in KeyValues.
//! - Sequences are weird. It's not possible to serialize top-level or nested sequences. See
//!   [`Error::UnrepresentableSequence`] for more.
//!
//! ## Missing Features
//!
//! This library is in an early state. As such, many features have not yet been implemented.
//! Some missing features include:
//!
//! - Deserialization
//!   - Text parsing
//!   - Conversion to Rust types
//! - An easier API for [`Object`]
//! - A `keyvalues!` macro to create [`Object`]s
//! - Conditional tags
//!   - The [`ser::Formatter`] API supports conditional tags, but this is unsupported for the
//!     serde API.
//! - `#base` and `#include` directives
//!   - The [`ser::Formatter`] API supports macro formatting, but the serde API treats
//!     macros like normal fields.

#![warn(missing_docs)]

mod de;
pub mod error;
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
