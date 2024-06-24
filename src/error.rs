use std::fmt::Display;
use std::io;
use thiserror::Error;

// TODO: this struct is incomplete and subject to change
/// Represents all errors that can occur during serialization and deserialization.
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
    /// - `i128` and `u128` - KeyValues only supports up to `i64` and `u64`. If you really need 
    ///   to save a number this big, try using a string instead.
    /// - `[u8]` (raw bytes) - KeyValues cannot save binary data. Convert it to a string first. 
    #[error("type `{0}` is not supported")]
    UnsupportedType(String),
    
    /// Indicates that multiple root-level keys were discovered, but the deserialization function
    /// does not permit this.
    /// 
    /// # Explanation
    /// 
    /// Traditional KeyValues implementations require that all data is stored in a root-level
    /// object with exactly one key. This library also assumes this to be the case. However,
    /// certain KeyValue-like formats DO permit the root object to have multiple keys—namely,
    /// the VMF (Valve map file) format. To allow the library to handle these formats, two families
    /// of ser/de functions are provided: *key-value functions* and *value functions*.
    ///
    /// Key-value functions like [crate::ser::kv_to_string] and [crate::de::kv_from_string] 
    /// operate on a single key-value pair. They are mainly intended for serializing and 
    /// deserializing KeyValues files.
    /// 
    /// Value functions like [crate::ser::to_string] and [crate::de::from_string] handle values
    /// directly, with no enclosing object. These functions can handle multiple root level keys,
    /// as well as incomplete files.
    /// 
    /// This error occurs when attempting to deserialize a file that contains multiple root-level
    /// keys with a value function. When this happens, either the file was malformed or you should 
    /// have used a value function. 
    #[error("tried to deserialize multiple root keys (try `from_str` or `from_reader`)")]
    MultipleRootKeys,
    
    /// Indicates that a sequence was serialized directly without an enclosing field.
    /// 
    /// # Explanation
    ///
    /// It is not possible to represent a sequence (such as a `Vec` or a tuple) as a singular, 
    /// standalone KeyValues value. For example, the following code panics: 
    /// 
    /// ```should_panic
    /// # use vdflex::ser::to_string;
    /// let nums = vec![1, 2, 3];
    /// let parsed = to_string(&nums).unwrap(); // panics
    /// ```
    /// 
    /// It might seem strange that this is an error. After all, JSON doesn't have an issue with 
    /// this! Neither does YAML, or XML, or pretty much any popular serialization format.
    ///
    /// Simply put, bare sequences are completely unrepresentable in KeyValues. This is because of 
    /// the way sequences are represented: by repeating the element multiple times.
    ///
    /// ```
    /// # use indoc::indoc;
    /// # use vdflex::ser::to_string;
    /// #[derive(serde::Serialize)]
    /// struct Data { nums: Vec<i32> }
    /// 
    /// let data = Data { nums: vec![1, 2, 3] };
    /// println!("My data is:\n{}", to_string(&data).unwrap());
    /// // My data is: 
    /// // "nums" "1"
    /// // "nums" "2"
    /// // "nums" "3"
    /// # assert_eq!(
    /// #    to_string(&data).unwrap(),
    /// #    indoc! {r#"
    /// #        "nums" "1"
    /// #        "nums" "2"
    /// #        "nums" "3"
    /// #    "#},
    /// # );
    /// ``` 
    /// 
    /// This means that there is no sensible way to represent top-level sequences. This also means
    /// that there is no way to represent [nested sequences](Error::NestedSequence).
    #[error("tried to serialize a sequence directly (try wrapping it in an object)")]
    RootLevelSequence,
    
    /// Indicates that a sequence containing another sequence was attempted to be serialized.
    /// 
    /// # Explanation
    /// 
    /// It is not possible to represent a sequence (such as a `Vec` or a tuple) that contains
    /// another sequence, unless there is some level of indirection (e.g. a struct). For example, 
    /// the following code fails:
    /// 
    /// ```should_panic
    /// # use vdflex::ser::to_string;
    /// let matrix = vec![vec![1, 0, 0], vec![0, 1, 0], vec![0, 0, 1]];
    /// let vdf = to_string(&matrix).unwrap(); // panics
    /// ```
    /// 
    /// This is because sequences are represented by repeating the parent element's key multiple
    /// times. The inner sequence(s) have no keys and thus no possible representation. For more
    /// information, see [Error::RootLevelSequence].
    #[error("tried to serialize a nested sequence")]
    NestedSequence,
    
    /// Indicates that a non-finite floating-point number was attempted to be serialized.
    ///
    /// # Explanation
    ///
    /// KeyValues has no way to represent non-finite floats (that is, infinity and NaN). 
    #[error("floating point value `{0}` is non-finite")]
    NonFiniteFloat(f64),
    
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

pub type Result<T> = std::result::Result<T, Error>;
