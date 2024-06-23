mod formatter;
mod serializer;

use crate::Result;
use serde::Serialize;
use std::collections::HashMap;
use std::io::Write;

pub use formatter::{BraceStyle, FormatOpts, Formatter, PrettyFormatter, Quoting};
pub use serializer::Serializer;

macro_rules! to_string_impl {
    ($to_writer:ident, $($arg:expr),* ) => {{
        let mut writer = Vec::new();
        $to_writer(&mut writer, $($arg),*)?;
        // Safety: given valid utf-8 as input, the writer will never produce invalid utf-8
        unsafe { Ok(String::from_utf8_unchecked(writer)) }
    }};
}

macro_rules! to_writer_impl {
    (@nested $serializer:expr, ($key:expr, $value:expr)) => {{
        let mut serializer = $serializer;
        let mut root = HashMap::new();
        root.insert($key, $value);
        root.serialize(&mut serializer)
    }};
    (@flat $serializer:expr, $value:expr) => {{
        let mut serializer = $serializer;
        $value.serialize(&mut serializer)
    }};
}

/// Serialize the given value as flattened KeyValues text.
///
/// # Errors
///
/// Serialization can fail if `T` cannot be represented as KeyValues or if `T`'s implementation
/// of `Serialize` decides to fail.
pub fn to_string<T: ?Sized + Serialize>(value: &T) -> Result<String> {
    to_string_impl!(to_writer, value)
}

/// Serialize the given value as flattened, pretty-printed KeyValues text.
///
/// # Errors
///
/// Serialization can fail if `T` cannot be represented as KeyValues or if `T`'s implementation
/// of `Serialize` decides to fail.
pub fn to_string_pretty<T: ?Sized + Serialize>(value: &T, opts: FormatOpts) -> Result<String> {
    to_string_impl!(to_writer_pretty, value, opts)
}

/// Serialize the given value as flattened KeyValues text with a custom formatter.
///
/// # Errors
///
/// Serialization can fail if `T` cannot be represented as KeyValues or if `T`'s implementation
/// of `Serialize` decides to fail.
pub fn to_string_custom<T: ?Sized + Serialize, F: Formatter>(
    value: &T,
    formatter: F,
) -> Result<String> {
    to_string_impl!(to_writer_custom, value, formatter)
}

/// Serialize the given value as KeyValues text using a specified root key.
///
/// # Errors
///
/// Serialization can fail if `T` cannot be represented as KeyValues or if `T`'s implementation
/// of `Serialize` decides to fail.
pub fn to_string_nested<T: ?Sized + Serialize>(key: &str, value: &T) -> Result<String> {
    to_string_impl!(to_writer_nested, key, value)
}

/// Serialize the given value as pretty-printed KeyValues text using a specified root key.
///
/// # Errors
///
/// Serialization can fail if `T` cannot be represented as KeyValues or if `T`'s implementation
/// of `Serialize` decides to fail.
pub fn to_string_nested_pretty<T: ?Sized + Serialize>(
    key: &str,
    value: &T,
    opts: FormatOpts,
) -> Result<String> {
    to_string_impl!(to_writer_nested_pretty, key, value, opts)
}

/// Serialize the given value as KeyValues text with a custom formatter using a specified root key.
///
/// # Errors
///
/// Serialization can fail if `T` cannot be represented as KeyValues or if `T`'s implementation
/// of `Serialize` decides to fail.
pub fn to_string_nested_custom<T: ?Sized + Serialize, F: Formatter>(
    key: &str,
    value: &T,
    formatter: F,
) -> Result<String> {
    to_string_impl!(to_writer_nested_custom, key, value, formatter)
}

/// Serialize the given value as flattened KeyValues into the specified writer.
///
/// # Errors
///
/// Serialization can fail if `T` cannot be represented as KeyValues or if `T`'s implementation
/// of `Serialize` decides to fail.
#[inline]
pub fn to_writer<W: Write, T: ?Sized + Serialize>(writer: W, value: &T) -> Result<()> {
    to_writer_impl!(@flat Serializer::new(writer), value)
}

/// Serialize the given value as flattened, pretty-printed KeyValues into the specified writer.
///
/// # Errors
///
/// Serialization can fail if `T` cannot be represented as KeyValues or if `T`'s implementation
/// of `Serialize` decides to fail.
pub fn to_writer_pretty<W: Write, T: ?Sized + Serialize>(
    writer: W,
    value: &T,
    opts: FormatOpts,
) -> Result<()> {
    to_writer_impl!(@flat Serializer::pretty(writer, opts), value)
}

/// Serialize the given value as flattened KeyValues with a custom formatter into the specified
/// writer.
///
/// # Errors
///
/// Serialization can fail if `T` cannot be represented as KeyValues or if `T`'s implementation
/// of `Serialize` decides to fail.
pub fn to_writer_custom<W: Write, T: ?Sized + Serialize, F: Formatter>(
    writer: W,
    value: &T,
    formatter: F,
) -> Result<()> {
    to_writer_impl!(@flat Serializer::custom(writer, formatter), value)
}

/// Serialize the given value into the specified writer with a specified root key.
///
/// # Errors
///
/// Serialization can fail if `T` cannot be represented as KeyValues or if `T`'s implementation
/// of `Serialize` decides to fail.
pub fn to_writer_nested<W: Write, T: ?Sized + Serialize>(
    writer: W,
    key: &str,
    value: &T,
) -> Result<()> {
    to_writer_impl!(@nested Serializer::new(writer), (key, value))
}

/// Serialize the given value as pretty-printed KeyValues into the specified writer with a
/// specified root key.
///
/// # Errors
///
/// Serialization can fail if `T` cannot be represented as KeyValues or if `T`'s implementation
/// of `Serialize` decides to fail.
pub fn to_writer_nested_pretty<W: Write, T: ?Sized + Serialize>(
    writer: W,
    key: &str,
    value: &T,
    opts: FormatOpts,
) -> Result<()> {
    to_writer_impl!(@nested Serializer::pretty(writer, opts), (key, value))
}

/// Serialize the given value as KeyValues with a custom formatter into the specified writer using a
/// specified root key.
///
/// # Errors
///
/// Serialization can fail if `T` cannot be represented as KeyValues or if `T`'s implementation
/// of `Serialize` decides to fail.
pub fn to_writer_nested_custom<W: Write, T: ?Sized + Serialize, F: Formatter>(
    writer: W,
    key: &str,
    value: &T,
    formatter: F,
) -> Result<()> {
    to_writer_impl!(@nested Serializer::custom(writer, formatter), (key, value))
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
    fn ser_simple() -> Result<()> {
        let boots = Cat {
            name: String::from("Boots"),
            age: 22,
            likes_catnip: true,
        };

        assert_eq!(
            to_string_nested("Cat", &boots)?,
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
    fn ser_simple_nested() -> Result<()> {
        let boots = Cat {
            name: String::from("Boots"),
            age: 22,
            likes_catnip: true,
        };

        assert_eq!(
            to_string_nested("Cat", &boots)?,
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
    fn ser_simple_flat() -> Result<()> {
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
    fn ser_simple_pretty() -> Result<()> {
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
            to_string_nested_pretty("Cat", &boots, opts)?,
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
