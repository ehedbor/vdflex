use std::io::{self, Write};

/// This trait allows the user to customize KeyValues formatting.
///
/// By default, there is only one implementation: [PrettyFormatter].
pub trait Formatter {
    /// Called before writing an object (except the root). Writes a `{` to the specified writer.
    fn begin_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write;

    /// Called after every object (except the root). Writes a `}` to the specified writer.
    fn end_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write;

    /// Writes an object key.
    fn write_key<W>(&mut self, writer: &mut W, key: &str) -> io::Result<()>
    where
        W: ?Sized + Write;

    /// Writes a macro key.
    fn write_macro_key<W>(&mut self, writer: &mut W, key: &str) -> io::Result<()>
    where
        W: ?Sized + Write;

    /// Writes a string value.
    fn write_value<W>(&mut self, writer: &mut W, s: &str) -> io::Result<()>
    where
        W: ?Sized + Write;

    /// Writes a conditional tag.
    fn write_conditional<W>(&mut self, writer: &mut W, condition: &str) -> io::Result<()>
    where
        W: ?Sized + Write;

    /// Writes a line comment.
    fn write_line_comment<W>(&mut self, writer: &mut W, comment: &str) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        write!(writer, "// {comment}\n")
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
}

impl PrettyFormatter {
    pub fn new() -> Self {
        Self::with_opts(FormatOpts::default())
    }

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

    fn write_quotable<W>(&mut self, writer: &mut W, s: &str, quoting: Quoting) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        // Write a quote if necessary and remember for later.
        let need_quotes = match quoting {
            Quoting::Always => true,
            Quoting::WhenRequired => {
                s.starts_with('[')
                    || s.contains(|c: char| c == '{' || c == '}' || c == '"' || c.is_whitespace())
            }
        };

        if need_quotes {
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

        // write the trailing quote
        if need_quotes {
            writer.write_all(b"\"")?;
        }

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

    fn write_key<W>(&mut self, writer: &mut W, key: &str) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.write_indent(writer)?;
        self.write_quotable(writer, key, self.opts.quote_keys)?;
        Ok(())
    }

    fn write_macro_key<W>(&mut self, writer: &mut W, key: &str) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.write_indent(writer)?;
        self.write_quotable(writer, key, self.opts.quote_macros)?;
        Ok(())
    }

    fn write_value<W>(&mut self, writer: &mut W, s: &str) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        writer.write_all(b" ")?;
        self.write_quotable(writer, s, self.opts.quote_strings)?;
        writer.write_all(b"\n")?;
        Ok(())
    }

    fn write_conditional<W>(&mut self, writer: &mut W, condition: &str) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        write!(writer, " [{condition}]")
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
        f.write_key(w, "LightmappedGeneric")?;

        f.begin_object(w)?;
        {
            f.write_key(w, "$basetexture")?;
            f.write_value(w, "coast\\shingle_01")?;

            f.write_key(w, "$surfaceprop")?;
            f.write_value(w, "gravel")?;
        }
        f.end_object(w)?;

        Ok(())
    }

    fn write_nested_vdf<F, W>(f: &mut F, w: &mut W) -> io::Result<()>
    where
        F: Formatter,
        W: ?Sized + Write,
    {
        f.write_line_comment(w, "Test comment")?;

        f.write_macro_key(w, "#base")?;
        f.write_value(w, "panelBase.res")?;

        f.write_key(w, "Resource/specificPanel.res")?;
        f.begin_object(w)?;
        {
            f.write_key(w, "Greeting")?;
            f.write_value(w, "Hello, \"Bob\"!")?;

            f.write_key(w, "Nested")?;
            f.begin_object(w)?;
            {
                f.write_key(w, "Object")?;
                f.write_value(w, "1")?;
            }
            f.end_object(w)?;
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
