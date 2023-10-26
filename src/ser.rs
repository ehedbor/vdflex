mod formatter;
mod serializer;

use crate::{KeyvaluesRoot, Result, RootKind};
use serde::Serialize;
use std::collections::HashMap;
use std::io::Write;

pub use formatter::{FormatOpts, Formatter, IndentStyle, PrettyFormatter};
pub use serializer::Serializer;

/// Serialize the given value as KeyValues text.
///
/// # Errors
///
/// Serialization can fail if `T` cannot be represented as KeyValues or if `T`'s implementation
/// of `Serialize` decides to fail.
pub fn to_string<T>(value: &T) -> Result<String>
where
    T: ?Sized + Serialize + KeyvaluesRoot,
{
    match T::kind() {
        RootKind::Nested(ref root_key) => to_string_nested(root_key, value),
        RootKind::Flattened => to_string_flat(value),
    }
}

/// Serialize the given value as pretty-printed KeyValues text.
///
/// # Errors
///
/// Serialization can fail if `T` cannot be represented as KeyValues or if `T`'s implementation
/// of `Serialize` decides to fail.
pub fn to_string_pretty<T>(value: &T, opts: FormatOpts) -> Result<String>
where
    T: ?Sized + Serialize + KeyvaluesRoot,
{
    match T::kind() {
        RootKind::Nested(ref root_key) => to_string_nested_pretty(root_key, value, opts),
        RootKind::Flattened => to_string_flat_pretty(value, opts),
    }
}

/// Serialize the given value as KeyValues text using a specified root key.
///
/// # Errors
///
/// Serialization can fail if `T` cannot be represented as KeyValues or if `T`'s implementation
/// of `Serialize` decides to fail.
#[inline]
pub fn to_string_nested<T>(key: &str, value: &T) -> Result<String>
where
    T: ?Sized + Serialize,
{
    to_string_nested_pretty(key, value, FormatOpts::default())
}

/// Serialize the given value as pretty-printed KeyValues text using a specified root key.
///
/// # Errors
///
/// Serialization can fail if `T` cannot be represented as KeyValues or if `T`'s implementation
/// of `Serialize` decides to fail.
pub fn to_string_nested_pretty<T>(key: &str, value: &T, opts: FormatOpts) -> Result<String>
where
    T: ?Sized + Serialize,
{
    let mut writer = Vec::new();
    to_writer_nested_pretty(&mut writer, key, value, opts)?;
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

/// Serialize the given value into the specified writer.
///
/// # Errors
///
/// Serialization can fail if `T` cannot be represented as KeyValues or if `T`'s implementation
/// of `Serialize` decides to fail.
pub fn to_writer<W, T>(writer: W, value: &T) -> Result<()>
where
    W: Write,
    T: ?Sized + Serialize + KeyvaluesRoot,
{
    match T::kind() {
        RootKind::Nested(ref root_key) => to_writer_nested(writer, root_key, value),
        RootKind::Flattened => to_writer_flat(writer, value),
    }
}

/// Serialize the given value as pretty-printed KeyValues into the specified writer.
///
/// # Errors
///
/// Serialization can fail if `T` cannot be represented as KeyValues or if `T`'s implementation
/// of `Serialize` decides to fail.
pub fn to_writer_pretty<W, T>(writer: W, value: &T, opts: FormatOpts) -> Result<()>
where
    W: Write,
    T: ?Sized + Serialize + KeyvaluesRoot,
{
    match T::kind() {
        RootKind::Nested(ref root_key) => to_writer_nested_pretty(writer, root_key, value, opts),
        RootKind::Flattened => to_writer_flat_pretty(writer, value, opts),
    }
}

/// Serialize the given value into the specified writer with a specified root key.
///
/// # Errors
///
/// Serialization can fail if `T` cannot be represented as KeyValues or if `T`'s implementation
/// of `Serialize` decides to fail.
#[inline]
pub fn to_writer_nested<W, T>(writer: W, key: &str, value: &T) -> Result<()>
where
    W: Write,
    T: ?Sized + Serialize,
{
    to_writer_nested_pretty(writer, key, value, FormatOpts::default())
}

/// Serialize the given value as pretty-printed KeyValues into the specified writer with a
/// specified root key.
///
/// # Errors
///
/// Serialization can fail if `T` cannot be represented as KeyValues or if `T`'s implementation
/// of `Serialize` decides to fail.
pub fn to_writer_nested_pretty<W, T>(
    writer: W,
    key: &str,
    value: &T,
    opts: FormatOpts,
) -> Result<()>
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

    impl KeyvaluesRoot for Cat {
        fn kind() -> RootKind {
            RootKind::Nested(String::from("Cat"))
        }
    }

    #[test]
    #[cfg(feature = "preserve_order")]
    fn ser_keyvalues() -> Result<()> {
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
        let kv = Keyvalues::new(String::from("Settings"), Value::Object(root));

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
            to_string(&boots)?,
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
    fn ser_simple_pretty() -> Result<()> {
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
