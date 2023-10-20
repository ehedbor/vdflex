pub mod formatter;
pub mod serializer;

use crate::Result;
use serde::Serialize;
use std::collections::HashMap;
use std::io::Write;

pub use formatter::{EscapeSequence, FormatOpts, Formatter, IndentStyle, PrettyFormatter};
pub use serializer::Serializer;

/// Serialize the given value as KeyValues text using a specified root key.
///
/// # Errors
///
/// Serialization can fail if `T` cannot be represented as KeyValues or if `T`'s implementation
/// of `Serialize` decides to fail.
#[inline]
pub fn to_string<T>(key: &str, value: &T) -> Result<String>
where
    T: ?Sized + Serialize,
{
    to_string_pretty(key, value, FormatOpts::default())
}

/// Serialize the given value as pretty-printed KeyValues text using a specified root key.
///
/// # Errors
///
/// Serialization can fail if `T` cannot be represented as KeyValues or if `T`'s implementation
/// of `Serialize` decides to fail.
pub fn to_string_pretty<T>(key: &str, value: &T, opts: FormatOpts) -> Result<String>
where
    T: ?Sized + Serialize,
{
    let mut writer = Vec::new();
    to_writer_pretty(&mut writer, key, value, opts)?;
    // Safety: given valid utf-8 as input, the writer will never produce invalid utf-8
    unsafe { Ok(String::from_utf8_unchecked(writer)) }
}

/// Serialize the given value as flattened KeyValues text.
///
/// # Errors
///
/// Serialization can fail if `T` cannot be represented as KeyValues or if `T`'s implementation
/// of `Serialize` decides to fail.
#[inline]
pub fn to_string_flat<T>(value: &T) -> Result<String>
where
    T: ?Sized + Serialize,
{
    to_string_flat_pretty(value, FormatOpts::default())
}

/// Serialize the given value as flattened, pretty-printed KeyValues text.
///
/// # Errors
///
/// Serialization can fail if `T` cannot be represented as KeyValues or if `T`'s implementation
/// of `Serialize` decides to fail.
pub fn to_string_flat_pretty<T>(value: &T, opts: FormatOpts) -> Result<String>
where
    T: ?Sized + Serialize,
{
    let mut writer = Vec::new();
    to_writer_flat_pretty(&mut writer, value, opts)?;
    // Safety: given valid utf-8 as input, the writer will never produce invalid utf-8
    unsafe { Ok(String::from_utf8_unchecked(writer)) }
}

/// Serialize the given value into the specified writer with a specified root key.
///
/// # Errors
///
/// Serialization can fail if `T` cannot be represented as KeyValues or if `T`'s implementation
/// of `Serialize` decides to fail.
#[inline]
pub fn to_writer<W, T>(writer: W, key: &str, value: &T) -> Result<()>
where
    W: Write,
    T: ?Sized + Serialize,
{
    to_writer_pretty(writer, key, value, FormatOpts::default())
}

/// Serialize the given value as pretty-printed KeyValues into the specified ¶writer with a
/// specified root key.
///
/// # Errors
///
/// Serialization can fail if `T` cannot be represented as KeyValues or if `T`'s implementation
/// of `Serialize` decides to fail.
pub fn to_writer_pretty<W, T>(writer: W, key: &str, value: &T, opts: FormatOpts) -> Result<()>
where
    W: Write,
    T: ?Sized + Serialize,
{
    let serializer = Serializer::pretty(writer, opts);
    let mut root = HashMap::new();
    root.insert(key, value);
    root.serialize(serializer)
}

/// Serialize the given value as flattened KeyValues into the specified writer.
///
/// # Errors
///
/// Serialization can fail if `T` cannot be represented as KeyValues or if `T`'s implementation
/// of `Serialize` decides to fail.
#[inline]
pub fn to_writer_flat<W, T>(writer: W, value: &T) -> Result<()>
where
    W: Write,
    T: ?Sized + Serialize,
{
    to_writer_flat_pretty(writer, value, FormatOpts::default())
}

/// Serialize the given value as flattened, pretty-printed KeyValues into the specified writer.
///
/// # Errors
///
/// Serialization can fail if `T` cannot be represented as KeyValues or if `T`'s implementation
/// of `Serialize` decides to fail.
pub fn to_writer_flat_pretty<W, T>(writer: W, value: &T, opts: FormatOpts) -> Result<()>
where
    W: Write,
    T: ?Sized + Serialize,
{
    let serializer = Serializer::pretty(writer, opts);
    value.serialize(serializer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[derive(Serialize)]
    #[serde(rename = "PascalCase")]
    struct Cat {
        name: String,
        age: i32,
        likes_catnip: bool,
    }

    #[test]
    fn simple() -> Result<()> {
        let boots = Cat {
            name: String::from("Boots"),
            age: 22,
            likes_catnip: true,
        };

        assert_eq!(
            to_string("Cat", &boots)?,
            indoc! {r#"
                "Cat" 
                {
                    "Name" "Boots"
                    "Age" "22"
                    "LikesCatnip" "1"
                }
            "#}
        );

        Ok(())
    }

    #[test]
    fn simple_flat() -> Result<()> {
        let boots = Cat {
            name: String::from("Boots"),
            age: 22,
            likes_catnip: true,
        };

        assert_eq!(
            to_string_flat(&boots)?,
            indoc! {r#"
                "Name" "Boots"
                "Age" "22"
                "LikesCatnip" "1"
            "#}
        );

        Ok(())
    }

    #[test]
    fn simple_pretty() -> Result<()> {
        let boots = Cat {
            name: String::from("Boots"),
            age: 22,
            likes_catnip: true,
        };

        let opts = FormatOpts {
            indent: String::from("  "),
            indent_style: IndentStyle::KAndR,
            ..Default::default()
        };

        assert_eq!(
            to_string_pretty("Cat", &boots, opts)?,
            indoc! {r#"
                "Cat" {
                  "Name" "Boots"
                  "Age" "22"
                  "LikesCatnip" "1"
                }
            "#}
        );

        Ok(())
    }
}
