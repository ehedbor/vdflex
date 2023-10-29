use std::fmt::Display;
use std::io;
use thiserror::Error;

// TODO: this struct is incomplete and subject to change
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("unexpected io error")]
    Io(io::Error),
    #[error("type `{0}` is not supported")]
    UnsupportedType(String),
    #[error("tried to deserialize multiple root keys (try `from_str_flat`?)")]
    MultipleRootKeys,
    #[error("tried to serialize a sequence directly (try wrapping it in an object)")]
    RootLevelSequence,
    #[error("floating point value `{0}` is non-finite")]
    NonFiniteFloat(f64),
    #[error("key must be a string, but it was a `{0}`")]
    KeyMustBeAString(String),
    // TODO: remove this and replace it with something better
    #[error("unsupported document key `{0}`")]
    UnsupportedKey(String),
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
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::Serde(msg.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
