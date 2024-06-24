//! (De)serialization errors

use std::fmt::Display;
use std::io;
use thiserror::Error;

// TODO: this struct is incomplete and subject to change
/// Represents all errors that can occur during serialization or deserialization.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// Indicates that an IO error occurred while (de)serializing.
    #[error("unexpected io error")]
    Io(io::Error),
    
    /// Indicates that the given type is not supported.
    /// 
    /// # Explanation
    /// 
    /// Not all Rust types have a suitable KeyValues equivalent. Some types that can't be 
    /// represented are: 
    ///
    /// - `[u8]` (raw bytes) - KeyValues cannot save binary data.
    #[error("type `{0}` is not supported")]
    UnsupportedType(String),

    /// Indicates that multiple root-level keys were discovered, but the deserialization function
    /// used does not permit this.
    /// 
    /// # Explanation
    /// 
    /// Traditional KeyValues implementations require that all data is stored in a root-level
    /// object with exactly one key. This library also assumes this to be the case. However,
    /// certain KeyValue-like formats DO permit the root object to have multiple keys—namely,
    /// the VMF (Valve map file) format. To allow the library to handle these formats, two families
    /// of ser/de functions are provided: *key-value functions* and *value functions*.
    ///
    /// Key-value functions like [`crate::kv_to_string`] and `crate::kv_from_string`
    /// operate on a single key-value pair. They are mainly intended for serializing and
    /// deserializing KeyValues files.
    /// 
    /// Value functions like [`crate::to_string`] and `crate::from_string` handle values
    /// directly, with no enclosing object. These functions can handle multiple root level keys,
    /// as well as incomplete files.
    /// 
    /// This error occurs when attempting to deserialize a file that contains multiple root-level
    /// keys with a value function. When this happens, either the file was malformed or you should 
    /// have used a value function. 
    #[error("tried to deserialize multiple root keys (try `from_str` or `from_reader`)")]
    MultipleRootKeys,
    
    /// Indicates that an unrepresentable sequence was serialized.
    /// 
    /// # Explanation
    ///
    /// There are two cases in which sequences cannot be serialized. The first occurs when a
    /// top-level sequence is serialized, like so:
    ///
    /// ```
    /// # use vdflex::error::Error;
    /// # use vdflex::ser::to_string;
    /// let nums = [1, 2, 3];
    /// // println!(to_string(&nums).unwrap()); // panics
    /// # assert!(matches!(to_string(&nums), Err(Error::UnrepresentableSequence)));
    /// ```
    ///
    /// The second case happens when a nested sequence is serialized:
    ///
    /// ```
    /// # use vdflex::error::Error;
    /// # use vdflex::ser::to_string;
    /// let nested = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];
    /// // println!(to_string(&nested).unwrap()); // panics
    /// # assert!(matches!(to_string(&nested), Err(Error::UnrepresentableSequence)));
    /// ```
    ///
    /// These errors both have the same root cause: the representation of sequences in KeyValues.
    /// In short, sequences are represented by repeating the parent element's key for each value
    /// in the sequence.
    ///
    /// ```
    /// # use indoc::indoc;
    /// # use vdflex::ser::{to_string, kv_to_string};
    /// #[derive(serde::Serialize)]
    /// struct Data { nums: Vec<i32> }
    /// 
    /// let data = Data { nums: vec![1, 2, 3] };
    /// println!("{}", kv_to_string("Data", &data).unwrap());
    /// // This prints the following:
    /// // Data
    /// // {
    /// //     "nums" "1"
    /// //     "nums" "2"
    /// //     "nums" "3"
    /// // }
    /// # assert_eq!(
    /// #    to_string(&data).unwrap(),
    /// #    indoc! {r#"
    /// #        "nums" "1"
    /// #        "nums" "2"
    /// #        "nums" "3"
    /// #    "#},
    /// # );
    ///
    /// let empty_data = Data { nums: vec![] };
    /// println!("{}", kv_to_string("Data", &empty_data).unwrap());
    /// // Data
    /// // {
    /// // }
    /// # assert_eq!(to_string(&empty_data).unwrap(), "");
    /// ```
    /// 
    /// As a result, sequences must be direct children of stuff with keys (e.g. maps and structs).
    #[error("tried to serialize a sequence with no valid KeyValues representation")]
    UnrepresentableSequence,

    /// Indicates that something that was not representable as a string was used as an
    /// object key.
    ///
    /// # Explanation
    ///
    /// Object keys are always strings in KeyValues. It is not possible to use data types
    /// without a string representation as keys.
    #[error("key must be a string, but it was a `{0}`")]
    KeyMustBeAString(String),

    /// Indicates that a Serde error occurred.
    #[error("a serde error occurred: {0}")]
    Serde(String),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::Io(value)
    }
}

impl serde::ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Serde(msg.to_string())
    }
}

impl serde::de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Serde(msg.to_string())
    }
}

/// Type alias for [Result](std::result::Result) with [enum@Error] as the error type.
pub type Result<T> = std::result::Result<T, Error>;
