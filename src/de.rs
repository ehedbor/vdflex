use crate::{Error, Result};
use serde::de::DeserializeOwned;
use serde::Deserialize;

pub fn from_str<'a, T>(s: &'a str) -> Result<T>
where
    T: Deserialize<'a>,
{
    todo!()
}

#[cfg(feature = "std")]
pub fn from_reader<R, T>(reader: R) -> Result<T>
where
    R: std::io::Read,
    T: DeserializeOwned,
{
    todo!()
}
