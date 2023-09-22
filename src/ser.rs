pub mod formatter;
pub mod serializer;

use crate::{Result, Value};
use serde::Serialize;
use std::io::Write;
use serde::ser::SerializeMap;

pub use formatter::{EscapeSequence, FormatOpts, Formatter, IndentStyle, PrettyFormatter};
pub use serializer::Serializer;

pub fn to_string<T>(_value: &T) -> Result<String>
where
    T: Serialize,
{
    todo!()
}

pub fn to_string_pretty<T>(_value: &T, _opts: &FormatOpts) -> Result<String>
where
    T: Serialize,
{
    todo!()
}

pub fn to_string_custom<F, T>(_formatter: &mut F, _value: &T) -> Result<String>
where
    F: Formatter,
    T: Serialize,
{
    todo!()
}

pub fn to_value<T>(_value: &T) -> Result<Value>
where
    T: Serialize,
{
    todo!()
}

pub fn to_write<W: Write, T: ?Sized + Serialize>(writer: W, value: &T) -> Result<()> {
    let mut serializer = Serializer::new(writer);
    serializer.serialize_value(value)
}

pub fn to_writer<W, T>(writer: W, value: &T) -> Result<()>
where
    W: Write,
    T: ?Sized + Serialize,
{
    let mut serializer = Serializer::new(writer);
    serializer.serialize_value(value)
}

pub fn to_writer_pretty<W, T>(writer: W, value: &T, opts: FormatOpts) -> Result<()>
where
    W: Write,
    T: Serialize,
{
    let mut serializer = Serializer::pretty(writer, opts);
    serializer.serialize_value(value)
}

pub fn to_writer_custom<W, F, T>(
    writer: W,
    formatter: F,
    value: &T,
) -> Result<()>
where
    W: Write,
    F: Formatter,
    T: Serialize,
{
    let mut serializer = Serializer::custom(writer, formatter);
    serializer.serialize_value(value)
}
