//! Serialize Rust types to KeyValues text.

mod formatter;
mod serializer;

use crate::Result;
use serde::Serialize;
use std::collections::HashMap;
use std::io::Write;

pub use formatter::{BraceStyle, FormatOpts, Formatter, PrettyFormatter, Quoting};
pub use serializer::Serializer;

/// Serialize the given value as a KeyValues value.
///
/// # Errors
///
/// Serialization can fail if `T` cannot be represented as KeyValues or if `T`'s implementation
/// of `Serialize` decides to fail.
#[cfg_attr(not(debug_assertions), inline(always))]
pub fn to_string<T: ?Sized + Serialize>(value: &T) -> Result<String> {
    to_string_pretty(value, PrettyFormatter::default())
}

/// Serialize the given value as a KeyValues value using a custom formatter.
///
/// # Errors
///
/// Serialization can fail if `T` cannot be represented as KeyValues or if `T`'s implementation
/// of `Serialize` decides to fail.
pub fn to_string_pretty<T: ?Sized + Serialize, F: Formatter>(
    value: &T,
    formatter: F,
) -> Result<String> {
    let mut writer = Vec::new();
    to_writer_pretty(&mut writer, value, formatter)?;
    // Safety: given valid utf-8 as input, the writer will never produce invalid utf-8
    unsafe { Ok(String::from_utf8_unchecked(writer)) }
}

/// Serialize the given value as a KeyValues object with the specified root key.
///
/// # Errors
///
/// Serialization can fail if `T` cannot be represented as KeyValues or if `T`'s implementation
/// of `Serialize` decides to fail.
#[cfg_attr(not(debug_assertions), inline(always))]
pub fn kv_to_string<T: ?Sized + Serialize>(key: &str, value: &T) -> Result<String> {
    kv_to_string_pretty(key, value, PrettyFormatter::default())
}

/// Serialize the given value as a KeyValues object with the specified root key using a custom
/// formatter.
///
/// # Errors
///
/// Serialization can fail if `T` cannot be represented as KeyValues or if `T`'s implementation
/// of `Serialize` decides to fail.
pub fn kv_to_string_pretty<T: ?Sized + Serialize, F: Formatter>(
    key: &str,
    value: &T,
    formatter: F,
) -> Result<String> {
    let mut writer = Vec::new();
    kv_to_writer_pretty(&mut writer, key, value, formatter)?;
    // Safety: given valid utf-8 as input, the writer will never produce invalid utf-8
    unsafe { Ok(String::from_utf8_unchecked(writer)) }
}

/// Serialize the given value as a KeyValues value into the specified writer.
///
/// # Errors
///
/// Serialization can fail if `T` cannot be represented as KeyValues or if `T`'s implementation
/// of `Serialize` decides to fail.
#[inline(always)]
pub fn to_writer<W: Write, T: ?Sized + Serialize>(writer: W, value: &T) -> Result<()> {
    to_writer_pretty(writer, value, PrettyFormatter::default())
}

/// Serialize the given value as a KeyValues value into the specified writer using a custom formatter.
///
/// # Errors
///
/// Serialization can fail if `T` cannot be represented as KeyValues or if `T`'s implementation
/// of `Serialize` decides to fail.
pub fn to_writer_pretty<W: Write, T: ?Sized + Serialize, F: Formatter>(
    writer: W,
    value: &T,
    formatter: F,
) -> Result<()> {
    let mut serializer = Serializer::new(writer, formatter);
    value.serialize(&mut serializer)
}

/// Serialize the given value as a KeyValues object with the specified root key into the specified
/// writer.
///
/// # Errors
///
/// Serialization can fail if `T` cannot be represented as KeyValues or if `T`'s implementation
/// of `Serialize` decides to fail.
#[inline(always)]
pub fn kv_to_writer<W: Write, T: ?Sized + Serialize>(
    writer: W,
    key: &str,
    value: &T,
) -> Result<()> {
    kv_to_writer_pretty(writer, key, value, PrettyFormatter::default())
}

/// Serialize the given value as a KeyValues object with the specified root key into the specified
/// writer using a custom formatter.
///
/// # Errors
///
/// Serialization can fail if `T` cannot be represented as KeyValues or if `T`'s implementation
/// of `Serialize` decides to fail.
pub fn kv_to_writer_pretty<W: Write, T: ?Sized + Serialize, F: Formatter>(
    writer: W,
    key: &str,
    value: &T,
    formatter: F,
) -> Result<()> {
    let mut serializer = Serializer::new(writer, formatter);
    let mut root = HashMap::with_capacity(1);
    root.insert(key, value);
    root.serialize(&mut serializer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{KeyValues, Object, Value};
    use indoc::indoc;

    #[derive(Serialize)]
    #[serde(rename_all = "PascalCase")]
    struct Cat {
        name: String,
        age: i32,
        likes_catnip: bool,
    }

    #[test]
    #[cfg(feature = "preserve_order")]
    fn ser_key_values() -> Result<()> {
        let mut root = Object::new();
        root.insert(
            String::from("FieldOfView"),
            vec![Value::String(String::from("80.0"))],
        );
        root.insert(
            String::from("Sensitivity"),
            vec![Value::String(String::from("0.9"))],
        );
        root.insert(
            String::from("Volume"),
            vec![Value::String(String::from("0.15"))],
        );
        let kv = KeyValues::new(String::from("Settings"), Value::Object(root));

        assert_eq!(
            to_string(&kv)?,
            indoc! {r#"
                "Settings"
                {
                    "FieldOfView" "80.0"
                    "Sensitivity" "0.9"
                    "Volume" "0.15"
                }
            "#}
        );

        Ok(())
    }

    #[test]
    fn ser_value() -> Result<()> {
        let boots = Cat {
            name: String::from("Boots"),
            age: 22,
            likes_catnip: true,
        };

        assert_eq!(
            to_string(&boots)?,
            indoc! {r#"
                "Name" "Boots"
                "Age" "22"
                "LikesCatnip" "1"
            "#}
        );

        Ok(())
    }

    #[test]
    fn ser_key_value() -> Result<()> {
        let boots = Cat {
            name: String::from("Boots"),
            age: 22,
            likes_catnip: true,
        };

        assert_eq!(
            kv_to_string("Cat", &boots)?,
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
    fn ser_key_value_pretty() -> Result<()> {
        let boots = Cat {
            name: String::from("Boots"),
            age: 22,
            likes_catnip: true,
        };

        let opts = FormatOpts {
            indent: String::from("  "),
            brace_style: BraceStyle::KAndR,
            ..Default::default()
        };

        assert_eq!(
            kv_to_string_pretty("Cat", &boots, PrettyFormatter::new(opts))?,
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
