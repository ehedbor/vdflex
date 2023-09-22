use std::io::{self, Write};

pub enum EscapeSequence {
    NewLine,
    Tab,
    Backslash,
    Quote,
}

/// This trait allows the user to customize Keyvalues formatting.
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
