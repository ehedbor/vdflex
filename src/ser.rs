use crate::{Error, Result};
use serde::Serialize;
use std::io;
use std::io::Write;

pub struct Serializer<W, F>
where
    W: Write,
    F: Formatter,
{
    writer: W,
    formatter: F,
}

impl<W> Serializer<W, PrettyFormatter>
where
    W: Write,
{
    /// Creates a new Keyvalues serializer using an appropriate formatter.
    pub fn new(writer: W) -> Self {
        Self::with_formatter(writer, PrettyFormatter::default())
    }

    /// Creates a new Keyvalues serializer using a `PrettyFormatter` with the given options.
    pub fn pretty(writer: W, opts: FormatOpts) -> Self {
        Self::with_formatter(writer, PrettyFormatter::with_opts(opts))
    }
}

impl<W, F> Serializer<W, F>
where
    W: Write,
    F: Formatter,
{
    pub fn with_formatter(writer: W, formatter: F) -> Self {
        Self { writer, formatter }
    }
}

impl<W, F> serde::Serializer for Serializer<W, F>
where
    W: Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        let s = if v { "1" } else { "0" };
        self.serialize_str(s)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_i128(self, _v: i128) -> Result<Self::Ok> {
        Err(Error::UnsupportedType("i128".to_string()))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_u128(self, _v: u128) -> Result<Self::Ok> {
        Err(Error::UnsupportedType("u128".to_string()))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(mut self, v: &str) -> Result<Self::Ok> {
        self.formatter
            .begin_string(&mut self.writer)
            .map_err(|e| Error::Io(e))?;

        let mut start = 0;
        for (idx, to_escape) in v.match_indices(&['\t', '\n', '\\', '\"']) {
            // Write a raw string fragment if one was present.
            if start != idx {
                self.formatter
                    .write_fragment(&mut self.writer, &v[start..idx])
                    .map_err(|e| Error::Io(e))?;
            }

            // Now write the escape character.
            let escape = match to_escape.chars().next().unwrap() {
                '\t' => EscapeSequence::Tab,
                '\n' => EscapeSequence::NewLine,
                '\\' => EscapeSequence::Backslash,
                '\"' => EscapeSequence::Quote,
                c => panic!("unexpected escape character {c:?}"),
            };
            self.formatter
                .write_escape_sequence(&mut self.writer, escape)
                .map_err(|e| Error::Io(e))?;

            start = idx + to_escape.len();
        }

        // If there was a trailing fragment, write that too.
        if start < v.len() {
            self.formatter
                .write_fragment(&mut self.writer, &v[start..])
                .map_err(|e| Error::Io(e))?;
        }

        self.formatter
            .end_string(&mut self.writer)
            .map_err(|e| Error::Io(e))?;

        Ok(())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok> {
        Err(Error::UnsupportedType("bytes".to_string()))
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        self.serialize_str("")
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Err(Error::UnsupportedType("unit".to_string()))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        Err(Error::UnsupportedType("unit struct".to_string()))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        Err(Error::UnsupportedType("enum newtype variant".to_string()))
        // self.formatter
        //     .begin_object(&mut self.writer)
        //     .map_err(|e| Error::Io(e))?;
        // {
        //     self.formatter
        //         .begin_key(&mut self.writer)
        //         .map_err(|e| Error::Io(e))?;
        //     self.serialize_str(variant)?;
        //     self.formatter
        //         .end_key(&mut self.writer)
        //         .map_err(|e| Error::Io(e))?;
        //
        //     value.serialize(self)?;
        // }
        // self.formatter
        //     .end_object(&mut self.writer)
        //     .map_err(|e| Error::Io(e))?;
        //
        // Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(Error::UnsupportedType("sequence".to_string()))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(Error::UnsupportedType("tuple".to_string()))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(Error::UnsupportedType("tuple struct".to_string()))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::UnsupportedType("enum tuple variant".to_string()))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::UnsupportedType("map".to_string()))
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(Error::UnsupportedType("struct".to_string()))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::UnsupportedType("enum struct variant".to_string()))
    }
}

impl<W, F> serde::ser::SerializeSeq for Serializer<W, F>
where
    W: Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::UnsupportedType("sequence".to_string()))
    }

    fn end(self) -> Result<Self::Ok> {
        Err(Error::UnsupportedType("sequence".to_string()))
    }
}

impl<W, F> serde::ser::SerializeTuple for Serializer<W, F>
where
    W: Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::UnsupportedType("tuple".to_string()))
    }

    fn end(self) -> Result<Self::Ok> {
        Err(Error::UnsupportedType("tuple".to_string()))
    }
}

impl<W, F> serde::ser::SerializeTupleStruct for Serializer<W, F>
where
    W: Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::UnsupportedType("tuple struct".to_string()))
    }

    fn end(self) -> Result<Self::Ok> {
        Err(Error::UnsupportedType("tuple struct".to_string()))
    }
}

impl<W, F> serde::ser::SerializeTupleVariant for Serializer<W, F>
where
    W: Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::UnsupportedType("enum tuple variant".to_string()))
    }

    fn end(self) -> Result<Self::Ok> {
        Err(Error::UnsupportedType("enum tuple variant".to_string()))
    }
}

impl<W, F> serde::ser::SerializeMap for Serializer<W, F>
where
    W: Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, _key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::UnsupportedType("map".to_string()))
    }

    fn serialize_value<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::UnsupportedType("map".to_string()))
    }

    fn end(self) -> Result<Self::Ok> {
        Err(Error::UnsupportedType("map".to_string()))
    }
}

impl<W, F> serde::ser::SerializeStruct for Serializer<W, F>
where
    W: Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::UnsupportedType("struct".to_string()))
    }

    fn end(self) -> Result<Self::Ok> {
        Err(Error::UnsupportedType("struct".to_string()))
    }
}

impl<W, F> serde::ser::SerializeStructVariant for Serializer<W, F>
where
    W: Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::UnsupportedType("enum struct variant".to_string()))
    }

    fn end(self) -> Result<Self::Ok> {
        Err(Error::UnsupportedType("enum struct variant".to_string()))
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EscapeSequence {
    NewLine,
    Tab,
    Backslash,
    Quote,
}

/// Trait to customize keyvalues formatting.
pub trait Formatter {
    /// Called before writing an object. Writes a `{` to the specified writer.
    fn begin_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write;

    /// Called after every object. Writes a `}` to the specified writer.
    fn end_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write;

    /// Called before writing a key. Writes a `"` to the specified writer.
    fn begin_key<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write;

    /// Called after writing a key. Writes a `"` to the specified writer.
    fn end_key<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write;

    /// Called before writing a macro name.
    fn begin_macro_key<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write;

    /// Called after writing a macro name.
    fn end_macro_key<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write;

    /// Called before every string value. Writes a `"` to the specified writer.
    fn begin_string<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write;

    /// Called after every string value. Writes a `"` to the specified writer.
    fn end_string<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write;

    /// Called before writing a conditional. Writes a `[` to the specified writer.
    fn begin_conditional<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write;

    /// Called after writing a conditional. Writes a `]` to the specified writer.
    fn end_conditional<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write;

    /// Called before writing a line comment.
    fn begin_line_comment<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        writer.write_all(b"//")
    }

    /// Called after writing a line comment.
    fn end_line_comment<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        writer.write_all(b"\n")
    }

    /// Writes an escape sequence to the specified writer.
    fn write_escape_sequence<W>(&mut self, writer: &mut W, escape: EscapeSequence) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        match escape {
            EscapeSequence::NewLine => writer.write_all(b"\\n"),
            EscapeSequence::Tab => writer.write_all(b"\\t"),
            EscapeSequence::Backslash => writer.write_all(b"\\\\"),
            EscapeSequence::Quote => writer.write_all(b"\\\""),
        }
    }

    /// Writes a raw sequence of characters to the specified writer.
    fn write_fragment<W>(&mut self, writer: &mut W, fragment: &str) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        writer.write_all(fragment.as_bytes())
    }
}

#[derive(Copy, Clone, Debug)]
pub enum IndentStyle {
    /// Place `{` and `}` on new lines.
    ///
    /// # Examples
    ///
    /// ```plaintext
    /// "Object"
    /// {
    ///     "Key" "Value"
    /// }
    /// ```
    Allman,

    /// Place `{` on the same line, and `}` on the next line.
    ///
    /// # Examples
    ///
    /// ```plaintext
    /// "Object" {
    ///     "Key" "Value"
    /// }
    /// ```
    KAndR,
}

/// Controls if strings should be quoted or not.
#[derive(Copy, Clone, Debug)]
pub enum Quoting {
    /// Always add quotes.
    Always,
    // TODO: support optional quotes
    // /// Only add quotes when required. This happens if any of the following is true:
    // ///
    // /// 1. The string contains whitespace.
    // /// 2. The string contains one of the control characters (`{`, `}`, or `"`).
    // /// 3. The string begins with '[' (which starts a conditional).
    // WhenRequired,
}

#[derive(Clone, Debug)]
pub struct FormatOpts {
    pub indent: String,
    pub indent_style: IndentStyle,
    pub quote_keys: Quoting,
    pub quote_strings: Quoting,
    pub quote_macros: Quoting,
}

impl Default for FormatOpts {
    fn default() -> Self {
        FormatOpts {
            indent: String::from("    "),
            indent_style: IndentStyle::Allman,
            quote_keys: Quoting::Always,
            quote_strings: Quoting::Always,
            quote_macros: Quoting::Always,
        }
    }
}

pub struct PrettyFormatter {
    opts: FormatOpts,
    indent_level: i32,
}

impl PrettyFormatter {
    pub fn with_opts(opts: FormatOpts) -> Self {
        Self {
            opts,
            indent_level: 0,
        }
    }

    fn push_indent(&mut self) {
        self.indent_level += 1
    }

    fn pop_indent(&mut self) {
        debug_assert!(
            self.indent_level > 0,
            "indent was popped more than it was pushed!"
        );
        self.indent_level -= 1
    }

    fn write_indent<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        for _ in 0..self.indent_level {
            writer.write_all(self.opts.indent.as_bytes())?;
        }
        Ok(())
    }
}

impl Default for PrettyFormatter {
    fn default() -> Self {
        Self {
            opts: Default::default(),
            indent_level: 0,
        }
    }
}

impl Formatter for PrettyFormatter {
    fn begin_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        match self.opts.indent_style {
            IndentStyle::Allman => {
                writer.write_all(b"\n")?;
                self.write_indent(writer)?;
                writer.write_all(b"{")?;
                self.push_indent();
                writer.write_all(b"\n")?;
            }
            IndentStyle::KAndR => {
                writer.write_all(b" {")?;
                self.push_indent();
                writer.write_all(b"\n")?;
            }
        }

        Ok(())
    }

    fn end_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.pop_indent();
        self.write_indent(writer)?;
        writer.write_all(b"}\n")
    }

    fn begin_key<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.write_indent(writer)?;
        match self.opts.quote_keys {
            Quoting::Always => writer.write_all(b"\""),
        }
    }

    fn end_key<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        match self.opts.quote_keys {
            Quoting::Always => writer.write_all(b"\""),
        }
    }

    fn begin_macro_key<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.write_indent(writer)?;
        match self.opts.quote_macros {
            Quoting::Always => writer.write_all(b"\""),
        }
    }

    fn end_macro_key<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        match self.opts.quote_macros {
            Quoting::Always => writer.write_all(b"\""),
        }
    }

    fn begin_string<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        writer.write_all(b" ")?;
        match self.opts.quote_strings {
            Quoting::Always => writer.write_all(b"\""),
        }
    }

    fn end_string<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        match self.opts.quote_strings {
            Quoting::Always => writer.write_all(b"\"")?,
        }
        writer.write_all(b"\n")
    }

    fn begin_conditional<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        writer.write_all(b" [")
    }

    fn end_conditional<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        writer.write_all(b"]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use std::io;

    fn write_simple_vmt<F, W>(f: &mut F, w: &mut W) -> io::Result<()>
    where
        F: Formatter,
        W: ?Sized + Write,
    {
        f.begin_key(w)?;
        f.write_fragment(w, "LightmappedGeneric")?;
        f.end_key(w)?;

        f.begin_object(w)?;
        {
            f.begin_key(w)?;
            f.write_fragment(w, "$basetexture")?;
            f.end_key(w)?;

            f.begin_string(w)?;
            f.write_fragment(w, "coast\\shingle_01")?;
            f.end_string(w)?;

            f.begin_key(w)?;
            f.write_fragment(w, "$surfaceprop")?;
            f.end_key(w)?;

            f.begin_string(w)?;
            f.write_fragment(w, "gravel")?;
            f.end_string(w)?;
        }
        f.end_object(w)?;

        Ok(())
    }

    fn write_nested_vdf<F, W>(f: &mut F, w: &mut W) -> io::Result<()>
    where
        F: Formatter,
        W: ?Sized + Write,
    {
        f.begin_line_comment(w)?;
        f.write_fragment(w, " Test comment")?;
        f.end_line_comment(w)?;

        f.begin_macro_key(w)?;
        f.write_fragment(w, "#base")?;
        f.end_macro_key(w)?;

        f.begin_string(w)?;
        f.write_fragment(w, "panelBase.res")?;
        f.end_string(w)?;

        f.begin_key(w)?;
        f.write_fragment(w, "Resource/specificPanel.res")?;
        f.end_key(w)?;

        f.begin_object(w)?;
        {
            {
                f.begin_key(w)?;
                f.write_fragment(w, "Greeting")?;
                f.end_key(w)?;

                f.begin_string(w)?;
                f.write_fragment(w, "Hello, ")?;
                f.write_escape_sequence(w, EscapeSequence::Quote)?;
                f.write_fragment(w, "Bob")?;
                f.write_escape_sequence(w, EscapeSequence::Quote)?;
                f.write_fragment(w, "!")?;
                f.end_string(w)?;
            }

            {
                f.begin_key(w)?;
                f.write_fragment(w, "Nested")?;
                f.end_key(w)?;

                f.begin_object(w)?;
                {
                    f.begin_key(w)?;
                    f.write_fragment(w, "Object")?;
                    f.end_key(w)?;

                    f.begin_string(w)?;
                    f.write_fragment(w, "1")?;
                    f.end_string(w)?;
                }
                f.end_object(w)?;
            }
        }
        f.end_object(w)?;

        Ok(())
    }

    #[test]
    fn simple() {
        let mut f = PrettyFormatter::with_opts(FormatOpts {
            indent: "    ".to_string(),
            indent_style: IndentStyle::Allman,
            quote_keys: Quoting::Always,
            quote_strings: Quoting::Always,
            quote_macros: Quoting::Always,
        });
        let mut buf = Vec::new();
        write_simple_vmt(&mut f, &mut buf).unwrap();

        assert_eq!(
            String::from_utf8(buf).unwrap(),
            indoc! {r##"
                "LightmappedGeneric"
                {
                    "$basetexture" "coast\shingle_01"
                    "$surfaceprop" "gravel"
                }
            "##}
        );
    }

    #[test]
    fn simple_knr() {
        let mut f = PrettyFormatter::with_opts(FormatOpts {
            indent: "  ".to_string(),
            indent_style: IndentStyle::KAndR,
            quote_keys: Quoting::Always,
            quote_strings: Quoting::Always,
            quote_macros: Quoting::Always,
        });
        let mut buf = Vec::new();
        write_simple_vmt(&mut f, &mut buf).unwrap();

        assert_eq!(
            String::from_utf8(buf).unwrap(),
            indoc! {r##"
                "LightmappedGeneric" {
                  "$basetexture" "coast\shingle_01"
                  "$surfaceprop" "gravel"
                }
            "##}
        );
    }

    #[test]
    fn nested() -> io::Result<()> {
        let mut f = PrettyFormatter::with_opts(FormatOpts {
            indent: "    ".to_string(),
            indent_style: IndentStyle::Allman,
            quote_keys: Quoting::Always,
            quote_strings: Quoting::Always,
            quote_macros: Quoting::Always,
        });
        let mut buf = Vec::new();
        write_nested_vdf(&mut f, &mut buf)?;

        assert_eq!(
            String::from_utf8(buf).unwrap(),
            indoc! {r##"
                // Test comment
                "#base" "panelBase.res"
                "Resource/specificPanel.res"
                {
                    "Greeting" "Hello, \"Bob\"!"
                    "Nested"
                    {
                        "Object" "1"
                    }
                }
            "##}
        );

        Ok(())
    }

    #[test]
    fn nested_knr() -> io::Result<()> {
        let mut f = PrettyFormatter::with_opts(FormatOpts {
            indent: "  ".to_string(),
            indent_style: IndentStyle::KAndR,
            quote_keys: Quoting::Always,
            quote_strings: Quoting::Always,
            quote_macros: Quoting::Always,
        });
        let mut buf = Vec::new();
        write_nested_vdf(&mut f, &mut buf)?;

        assert_eq!(
            String::from_utf8(buf).unwrap(),
            indoc! {r##"
                // Test comment
                "#base" "panelBase.res"
                "Resource/specificPanel.res" {
                  "Greeting" "Hello, \"Bob\"!"
                  "Nested" {
                    "Object" "1"
                  }
                }
            "##}
        );

        Ok(())
    }
}
