use std::io::{self, Write};

/// This trait allows the user to customize KeyValues formatting.
///
/// By default, there is only one implementation: [PrettyFormatter].
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

    /// Writes a string that might be quoted and may need to be escaped.
    ///
    /// If you want to write an arbitrary string, see [Formatter::write_raw_str].
    ///
    /// # Safety
    ///
    /// This method may only be called between these methods:
    ///
    /// - [Formatter::begin_key] and [Formatter::end_key],
    /// - [Formatter::begin_macro_key] and [Formatter::end_macro_key], and
    /// - [Formatter::begin_string] and [Formatter::end_string].
    ///
    /// In addition, this method may only be called *once* between such a pair. This is due to a
    /// limitation of [PrettyFormatter]. In order to support optionally writing a quote, the
    /// formatter must know if a quote is necessary before writing any string fragments. However,
    /// this is only known once the entire fragment sequence is written. It was problematic to store
    /// references to the unwritten data without copying it, so the approach taken here is to
    /// abandon the idea of writing multiple fragments at once.
    fn write_quotable_str<W>(&mut self, writer: &mut W, s: &str) -> io::Result<()>
    where
        W: ?Sized + Write;

    /// Writes an entire string directly.
    fn write_raw_str<W>(&mut self, writer: &mut W, s: &str) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        writer.write_all(s.as_bytes())
    }
}

/// Controls the formatting of curly brackets in KeyValues objects.
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
    /// Only add quotes when required. This happens if any of the following is true:
    ///
    /// 1. The string contains whitespace.
    /// 2. The string contains one of the control characters (`{`, `}`, or `"`).
    /// 3. The string begins with '[' (which normally starts a conditional).
    WhenRequired,
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

/// A [Formatter] that prints a human-readable version of the input.
pub struct PrettyFormatter {
    opts: FormatOpts,
    indent_level: i32,
    force_quotes: bool,
}

impl PrettyFormatter {
    pub fn new() -> Self {
        Self::with_opts(FormatOpts::default())
    }

    pub fn with_opts(opts: FormatOpts) -> Self {
        Self {
            opts,
            indent_level: 0,
            force_quotes: false,
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

    fn start_quotable(&mut self, quoting: Quoting) -> io::Result<()> {
        self.force_quotes = match quoting {
            Quoting::Always => true,
            Quoting::WhenRequired => false,
        };
        Ok(())
    }

    fn end_quotable<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        if self.force_quotes {
            writer.write_all(b"\"")?;
        }
        self.force_quotes = false;
        Ok(())
    }
}

impl Default for PrettyFormatter {
    fn default() -> Self {
        Self::new()
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
        self.start_quotable(self.opts.quote_keys)
    }

    fn end_key<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.end_quotable(writer)
    }

    fn begin_macro_key<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.write_indent(writer)?;
        self.start_quotable(self.opts.quote_macros)
    }

    fn end_macro_key<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.end_quotable(writer)
    }

    fn begin_string<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        writer.write_all(b" ")?;
        self.start_quotable(self.opts.quote_strings)
    }

    fn end_string<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.end_quotable(writer)?;
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

    fn write_quotable_str<W>(&mut self, writer: &mut W, s: &str) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        // Write a quote if necessary and remember for later.
        let write_quote = if self.force_quotes {
            true
        } else {
            s.starts_with('[')
                || s.contains(|c: char| c == '{' || c == '}' || c == '"' || c.is_whitespace())
        };

        if write_quote {
            self.force_quotes = true;
            writer.write_all(b"\"")?;
        }

        // Write all fragment-escape pairs.
        let mut start = 0;
        for (current, unescaped) in s.match_indices(&['\t', '\n', '\\', '\"']) {
            // Write a raw string fragment if one was present.
            if start != current {
                writer.write_all(s[start..current].as_bytes())?;
            }

            // Now write the escape character.
            let escaped = match unescaped.chars().next().unwrap() {
                '\t' => "\\t",
                '\n' => "\\n",
                '\\' => "\\\\",
                '\"' => "\\\"",
                _ => unreachable!(),
            };
            writer.write_all(escaped.as_bytes())?;

            start = current + unescaped.len();
        }

        // If there was a trailing fragment, write that too.
        if start < s.len() {
            writer.write_all(s[start..].as_bytes())?;
        }

        Ok(())
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
        f.write_quotable_str(w, "LightmappedGeneric")?;
        f.end_key(w)?;

        f.begin_object(w)?;
        {
            f.begin_key(w)?;
            f.write_quotable_str(w, "$basetexture")?;
            f.end_key(w)?;

            f.begin_string(w)?;
            f.write_quotable_str(w, "coast\\shingle_01")?;
            f.end_string(w)?;

            f.begin_key(w)?;
            f.write_quotable_str(w, "$surfaceprop")?;
            f.end_key(w)?;

            f.begin_string(w)?;
            f.write_quotable_str(w, "gravel")?;
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
        f.write_raw_str(w, " Test comment")?;
        f.end_line_comment(w)?;

        f.begin_macro_key(w)?;
        f.write_quotable_str(w, "#base")?;
        f.end_macro_key(w)?;

        f.begin_string(w)?;
        f.write_quotable_str(w, "panelBase.res")?;
        f.end_string(w)?;

        f.begin_key(w)?;
        f.write_quotable_str(w, "Resource/specificPanel.res")?;
        f.end_key(w)?;

        f.begin_object(w)?;
        {
            {
                f.begin_key(w)?;
                f.write_quotable_str(w, "Greeting")?;
                f.end_key(w)?;

                f.begin_string(w)?;
                f.write_quotable_str(w, "Hello, \"Bob\"!")?;
                f.end_string(w)?;
            }

            {
                f.begin_key(w)?;
                f.write_quotable_str(w, "Nested")?;
                f.end_key(w)?;

                f.begin_object(w)?;
                {
                    f.begin_key(w)?;
                    f.write_quotable_str(w, "Object")?;
                    f.end_key(w)?;

                    f.begin_string(w)?;
                    f.write_quotable_str(w, "1")?;
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
            ..FormatOpts::default()
        });
        let mut buf = Vec::new();
        write_simple_vmt(&mut f, &mut buf).unwrap();

        assert_eq!(
            String::from_utf8(buf).unwrap(),
            indoc! {r##"
                "LightmappedGeneric"
                {
                    "$basetexture" "coast\\shingle_01"
                    "$surfaceprop" "gravel"
                }
            "##}
        );
    }

    #[test]
    fn simple_quote_keys() {
        let mut f = PrettyFormatter::with_opts(FormatOpts {
            indent: "    ".to_string(),
            indent_style: IndentStyle::Allman,
            quote_keys: Quoting::Always,
            quote_strings: Quoting::WhenRequired,
            ..FormatOpts::default()
        });
        let mut buf = Vec::new();
        write_simple_vmt(&mut f, &mut buf).unwrap();

        assert_eq!(
            String::from_utf8(buf).unwrap(),
            indoc! {r#"
                "LightmappedGeneric"
                {
                    "$basetexture" coast\\shingle_01
                    "$surfaceprop" gravel
                }
            "#}
        );
    }

    #[test]
    fn simple_quote_values() {
        let mut f = PrettyFormatter::with_opts(FormatOpts {
            indent: "    ".to_string(),
            indent_style: IndentStyle::Allman,
            quote_keys: Quoting::WhenRequired,
            quote_strings: Quoting::Always,
            ..FormatOpts::default()
        });
        let mut buf = Vec::new();
        write_simple_vmt(&mut f, &mut buf).unwrap();

        assert_eq!(
            String::from_utf8(buf).unwrap(),
            indoc! {r#"
                LightmappedGeneric
                {
                    $basetexture "coast\\shingle_01"
                    $surfaceprop "gravel"
                }
            "#}
        );
    }

    #[test]
    fn simple_knr() {
        let mut f = PrettyFormatter::with_opts(FormatOpts {
            indent: "  ".to_string(),
            indent_style: IndentStyle::KAndR,
            quote_keys: Quoting::Always,
            quote_strings: Quoting::Always,
            ..FormatOpts::default()
        });
        let mut buf = Vec::new();
        write_simple_vmt(&mut f, &mut buf).unwrap();

        assert_eq!(
            String::from_utf8(buf).unwrap(),
            indoc! {r#"
                "LightmappedGeneric" {
                  "$basetexture" "coast\\shingle_01"
                  "$surfaceprop" "gravel"
                }
            "#}
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
    fn nested_quote_keys() -> io::Result<()> {
        let mut f = PrettyFormatter::with_opts(FormatOpts {
            indent: "    ".to_string(),
            indent_style: IndentStyle::Allman,
            quote_keys: Quoting::Always,
            quote_strings: Quoting::WhenRequired,
            quote_macros: Quoting::WhenRequired,
        });
        let mut buf = Vec::new();
        write_nested_vdf(&mut f, &mut buf)?;

        assert_eq!(
            String::from_utf8(buf).unwrap(),
            indoc! {r##"
                // Test comment
                #base panelBase.res
                "Resource/specificPanel.res"
                {
                    "Greeting" "Hello, \"Bob\"!"
                    "Nested"
                    {
                        "Object" 1
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
