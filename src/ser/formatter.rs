use std::io::{self, Write};

/// This trait allows the user to customize KeyValues formatting.
///
/// By default, there is only one implementation: [PrettyFormatter].
pub trait Formatter {
    /// Called before writing an object (including the root).
    fn begin_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write;

    /// Called after every object (including the root).
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

    /// Writes a line comment. Must not be called while writing a key-value pair.
    fn write_line_comment<W>(&mut self, writer: &mut W, comment: &str) -> io::Result<()>
    where
        W: ?Sized + Write;
}

/// Controls the formatting of curly brackets in KeyValues objects.
#[derive(Copy, Clone, Debug)]
pub enum BraceStyle {
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
    /// The sequence of characters to print for each indent level.
    pub indent: String,
    /// The separator between keys and values.
    pub separator: String,
    /// How to format braces.
    pub brace_style: BraceStyle,
    /// How object keys should be quoted.
    pub quote_keys: Quoting,
    /// How macro keys should be quoted.
    pub quote_macro_keys: Quoting,
    /// How object/macro values should be quoted.
    pub quote_values: Quoting,
}

impl Default for FormatOpts {
    fn default() -> Self {
        FormatOpts {
            indent: String::from("    "),
            separator: String::from(" "),
            brace_style: BraceStyle::Allman,
            quote_keys: Quoting::Always,
            quote_macro_keys: Quoting::Always,
            quote_values: Quoting::Always,
        }
    }
}

const ROOT_INDENT: i32 = -1;

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
            indent_level: ROOT_INDENT,
        }
    }

    fn push_indent(&mut self) {
        self.indent_level += 1
    }

    fn pop_indent(&mut self) {
        debug_assert!(
            self.indent_level > ROOT_INDENT,
            "indent was popped more than it was pushed!"
        );
        self.indent_level -= 1
    }

    fn write_indent<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        debug_assert!(
            self.indent_level > ROOT_INDENT,
            "tried to write indent outside of root object"
        );

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
        if self.indent_level == ROOT_INDENT {
            self.push_indent();
        } else {
            match self.opts.brace_style {
                BraceStyle::Allman => {
                    writer.write_all(b"\n")?;
                    self.write_indent(writer)?;
                    writer.write_all(b"{")?;
                    self.push_indent();
                    writer.write_all(b"\n")?;
                }
                BraceStyle::KAndR => {
                    writer.write_all(b" {")?;
                    self.push_indent();
                    writer.write_all(b"\n")?;
                }
            }
        }
        Ok(())
    }

    fn end_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.pop_indent();
        if self.indent_level > ROOT_INDENT {
            self.write_indent(writer)?;
            writer.write_all(b"}\n")?;
        }
        Ok(())
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
        self.write_quotable(writer, key, self.opts.quote_macro_keys)?;
        Ok(())
    }

    fn write_value<W>(&mut self, writer: &mut W, s: &str) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        writer.write_all(self.opts.separator.as_bytes())?;
        self.write_quotable(writer, s, self.opts.quote_values)?;
        writer.write_all(b"\n")?;
        Ok(())
    }

    fn write_conditional<W>(&mut self, writer: &mut W, condition: &str) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        write!(writer, " [{condition}]")
    }

    fn write_line_comment<W>(&mut self, writer: &mut W, comment: &str) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        if self.indent_level > ROOT_INDENT {
            self.write_indent(writer)?;
        }
        writeln!(writer, "// {comment}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use std::error::Error;
    use std::io;

    fn write_simple_vmt<F, W>(f: &mut F, w: &mut W) -> io::Result<()>
    where
        F: Formatter,
        W: ?Sized + Write,
    {
        f.begin_object(w)?;
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

        f.begin_object(w)?;
        {
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
        }
        f.end_object(w)?;

        Ok(())
    }

    #[test]
    fn simple() -> Result<(), Box<dyn Error>> {
        let mut f = PrettyFormatter::with_opts(FormatOpts {
            indent: "    ".to_string(),
            brace_style: BraceStyle::Allman,
            quote_keys: Quoting::Always,
            quote_values: Quoting::Always,
            ..FormatOpts::default()
        });
        let mut buf = Vec::new();
        write_simple_vmt(&mut f, &mut buf)?;

        assert_eq!(
            String::from_utf8(buf)?,
            indoc! {r##"
                "LightmappedGeneric"
                {
                    "$basetexture" "coast\\shingle_01"
                    "$surfaceprop" "gravel"
                }
            "##}
        );
        Ok(())
    }

    #[test]
    fn simple_quote_keys() -> Result<(), Box<dyn Error>> {
        let mut f = PrettyFormatter::with_opts(FormatOpts {
            indent: "    ".to_string(),
            brace_style: BraceStyle::Allman,
            quote_keys: Quoting::Always,
            quote_values: Quoting::WhenRequired,
            ..FormatOpts::default()
        });
        let mut buf = Vec::new();
        write_simple_vmt(&mut f, &mut buf)?;

        assert_eq!(
            String::from_utf8(buf)?,
            indoc! {r#"
                "LightmappedGeneric"
                {
                    "$basetexture" coast\\shingle_01
                    "$surfaceprop" gravel
                }
            "#}
        );
        Ok(())
    }

    #[test]
    fn simple_quote_values() -> Result<(), Box<dyn Error>> {
        let mut f = PrettyFormatter::with_opts(FormatOpts {
            indent: "    ".to_string(),
            brace_style: BraceStyle::Allman,
            quote_keys: Quoting::WhenRequired,
            quote_values: Quoting::Always,
            ..FormatOpts::default()
        });
        let mut buf = Vec::new();
        write_simple_vmt(&mut f, &mut buf)?;

        assert_eq!(
            String::from_utf8(buf)?,
            indoc! {r#"
                LightmappedGeneric
                {
                    $basetexture "coast\\shingle_01"
                    $surfaceprop "gravel"
                }
            "#}
        );
        Ok(())
    }

    #[test]
    fn simple_yaml() -> Result<(), Box<dyn Error>> {
        let mut f = PrettyFormatter::with_opts(FormatOpts {
            indent: "  ".to_string(),
            separator: ": ".to_string(),
            brace_style: BraceStyle::KAndR,
            quote_keys: Quoting::WhenRequired,
            quote_values: Quoting::WhenRequired,
            ..FormatOpts::default()
        });
        let mut buf = Vec::new();
        write_simple_vmt(&mut f, &mut buf)?;

        // Developer's note: Please don't use this library to produce YAML.
        // Developer's note: This is not even valid YAML anyways.
        assert_eq!(
            String::from_utf8(buf)?,
            indoc! {r#"
                LightmappedGeneric {
                  $basetexture: coast\\shingle_01
                  $surfaceprop: gravel
                }
            "#}
        );
        Ok(())
    }

    #[test]
    fn simple_knr() -> Result<(), Box<dyn Error>> {
        let mut f = PrettyFormatter::with_opts(FormatOpts {
            indent: "  ".to_string(),
            brace_style: BraceStyle::KAndR,
            quote_keys: Quoting::Always,
            quote_values: Quoting::Always,
            ..FormatOpts::default()
        });
        let mut buf = Vec::new();
        write_simple_vmt(&mut f, &mut buf)?;

        assert_eq!(
            String::from_utf8(buf)?,
            indoc! {r#"
                "LightmappedGeneric" {
                  "$basetexture" "coast\\shingle_01"
                  "$surfaceprop" "gravel"
                }
            "#}
        );
        Ok(())
    }

    #[test]
    fn nested() -> Result<(), Box<dyn Error>> {
        let mut f = PrettyFormatter::with_opts(FormatOpts {
            indent: "    ".to_string(),
            separator: " ".to_string(),
            brace_style: BraceStyle::Allman,
            quote_macro_keys: Quoting::Always,
            quote_keys: Quoting::Always,
            quote_values: Quoting::Always,
        });
        let mut buf = Vec::new();
        write_nested_vdf(&mut f, &mut buf)?;

        assert_eq!(
            String::from_utf8(buf)?,
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
    fn nested_quote_keys() -> Result<(), Box<dyn Error>> {
        let mut f = PrettyFormatter::with_opts(FormatOpts {
            indent: "    ".to_string(),
            separator: " ".to_string(),
            brace_style: BraceStyle::Allman,
            quote_keys: Quoting::Always,
            quote_values: Quoting::WhenRequired,
            quote_macro_keys: Quoting::WhenRequired,
        });
        let mut buf = Vec::new();
        write_nested_vdf(&mut f, &mut buf)?;

        assert_eq!(
            String::from_utf8(buf)?,
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
    fn nested_knr() -> Result<(), Box<dyn Error>> {
        let mut f = PrettyFormatter::with_opts(FormatOpts {
            indent: "  ".to_string(),
            separator: " ".to_string(),
            brace_style: BraceStyle::KAndR,
            quote_keys: Quoting::Always,
            quote_values: Quoting::Always,
            quote_macro_keys: Quoting::Always,
        });
        let mut buf = Vec::new();
        write_nested_vdf(&mut f, &mut buf)?;

        assert_eq!(
            String::from_utf8(buf)?,
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

    #[test]
    fn nested_tab_stops() -> Result<(), Box<dyn Error>> {
        let mut f = PrettyFormatter::with_opts(FormatOpts {
            indent: "\t".to_string(),
            separator: "\t\t".to_string(),
            brace_style: BraceStyle::KAndR,
            quote_keys: Quoting::Always,
            quote_values: Quoting::Always,
            quote_macro_keys: Quoting::Always,
        });
        let mut buf = Vec::new();
        write_nested_vdf(&mut f, &mut buf)?;
        assert_eq!(
            String::from_utf8(buf)?,
            indoc! {"
                // Test comment
                \"#base\"\t\t\"panelBase.res\"
                \"Resource/specificPanel.res\" {
                \t\"Greeting\"\t\t\"Hello, \\\"Bob\\\"!\"
                \t\"Nested\" {
                \t\t\"Object\"\t\t\"1\"
                \t}
                }
            "}
        );
        Ok(())
    }
}
