use std::fmt::Display;
use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("unexpected io error")]
    Io(io::Error),
    #[error("type `{0}` is not supported")]
    UnsupportedType(String),
    #[error("a serde error occured")]
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
    fn custom<T>(msg: T) -> Self where T: Display {
        Error::Serde(msg.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
