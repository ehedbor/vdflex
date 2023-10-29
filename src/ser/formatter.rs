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

    /// Called before writing a key in a key-value pair.
    fn begin_key<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write;

    /// Called after writing a key in a key-value pair.
    fn end_key<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write;

    /// Called before writing a value in a key-value pair.
    fn begin_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write;

    /// Called after writing a value in a key-value pair.
    fn end_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write;

    /// Writes a string value.
    fn write_string<W>(&mut self, writer: &mut W, s: &str) -> io::Result<()>
    where
        W: ?Sized + Write;

    /// Writes a conditional tag. Must be called after `write_key` and before `end_key`.
    fn write_conditional<W>(&mut self, writer: &mut W, condition: &str) -> io::Result<()>
    where
        W: ?Sized + Write;

    /// Writes a line comment. Must not be called while writing a key-value pair.
    fn write_line_comment<W>(&mut self, writer: &mut W, comment: &str) -> io::Result<()>
    where
        W: ?Sized + Write;
}

/// Controls the formatting of curly brackets in KeyValues objects.
#[derive(Copy, Clone, Debug, PartialEq)]
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
#[derive(Copy, Clone, Debug, PartialEq)]
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

#[derive(Copy, Clone, Debug, PartialEq)]
enum ElementKind {
    Object,
    KeyValue,
    Key,
    Value,
}

/// A [Formatter] that prints a human-readable version of the input.
pub struct PrettyFormatter {
    opts: FormatOpts,
    elements: Vec<ElementKind>,
    indent_level: i32,
}

impl PrettyFormatter {
    pub fn new() -> Self {
        Self::with_opts(FormatOpts::default())
    }

    pub fn with_opts(opts: FormatOpts) -> Self {
        Self {
            opts,
            elements: Vec::new(),
            indent_level: -1,
        }
    }

    fn push_element(&mut self, kind: ElementKind) {
        self.elements.push(kind);
        if kind == ElementKind::Object {
            self.indent_level += 1;
        }
    }

    fn pop_element(&mut self) -> ElementKind {
        let elem = self.elements.pop().expect("attempted to pop root element");
        if elem == ElementKind::Object {
            self.indent_level -= 1;
        }
        return elem;
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

    fn write_string_element<W>(
        &mut self,
        writer: &mut W,
        s: &str,
        quoting: Quoting,
    ) -> io::Result<()>
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
        if self.elements.is_empty() {
            self.push_element(ElementKind::Object);
            return Ok(());
        }

        match self.opts.brace_style {
            BraceStyle::Allman => {
                writer.write_all(b"\n")?;
                self.write_indent(writer)?;
                writer.write_all(b"{")?;
                self.push_element(ElementKind::Object);
                writer.write_all(b"\n")?;
            }
            BraceStyle::KAndR => {
                writer.write_all(b" {")?;
                self.push_element(ElementKind::Object);
                writer.write_all(b"\n")?;
            }
        }
        Ok(())
    }

    fn end_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        let elem = self.pop_element();
        debug_assert_eq!(
            elem,
            ElementKind::Object,
            "attempted to end object before starting it"
        );

        if !self.elements.is_empty() {
            self.write_indent(writer)?;
            writer.write_all(b"}")?;
        }
        Ok(())
    }

    fn begin_key<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        self.push_element(ElementKind::KeyValue);
        self.push_element(ElementKind::Key);
        self.write_indent(writer)?;
        Ok(())
    }

    fn end_key<W>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        let elem = self.pop_element();
        debug_assert_eq!(
            elem,
            ElementKind::Key,
            "tried to end key before starting it"
        );
        debug_assert_eq!(
            self.elements.last(),
            Some(&ElementKind::KeyValue),
            "tried to end key before starting key-value (impossible?)"
        );

        Ok(())
    }

    fn begin_value<W>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        debug_assert_eq!(
            self.elements.last(),
            Some(&ElementKind::KeyValue),
            "tried to begin value before ending key"
        );
        self.push_element(ElementKind::Value);
        // Don't write the separator yet; values can be objects as well
        Ok(())
    }

    fn end_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        let elem = self.pop_element();
        debug_assert_eq!(
            elem,
            ElementKind::Value,
            "tried to end value before beginning it"
        );

        let elem = self.pop_element();
        debug_assert_eq!(
            elem,
            ElementKind::KeyValue,
            "tried to end value before beginning key-value (impossible?)"
        );

        writer.write_all(b"\n")
    }

    fn write_string<W>(&mut self, writer: &mut W, s: &str) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        let element = self.elements.last();
        debug_assert_ne!(
            element,
            Some(&ElementKind::Object),
            "tried to write string directly to object"
        );
        debug_assert_ne!(
            element,
            Some(&ElementKind::KeyValue),
            "tried to write string directly to key-value pair"
        );

        let quoting = match self.elements.last() {
            Some(ElementKind::Key) => {
                if s == "#include" || s == "#base" {
                    self.opts.quote_macro_keys
                } else {
                    self.opts.quote_keys
                }
            }
            Some(ElementKind::Value) => self.opts.quote_values,
            // Allow serializing plain values
            _ => self.opts.quote_values,
        };

        if element == Some(&ElementKind::Value) {
            writer.write_all(self.opts.separator.as_bytes())?;
        }

        self.write_string_element(writer, s, quoting)
    }

    fn write_conditional<W>(&mut self, writer: &mut W, condition: &str) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        debug_assert_eq!(
            self.elements.last(),
            Some(&ElementKind::Key),
            "tried to write conditional tag outside of a key"
        );
        write!(writer, " [{condition}]")
    }

    fn write_line_comment<W>(&mut self, writer: &mut W, comment: &str) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        debug_assert_ne!(
            self.elements.last(),
            Some(&ElementKind::KeyValue),
            "tried to write line comment in a key-value pair"
        );
        debug_assert_ne!(
            self.elements.last(),
            Some(&ElementKind::Key),
            "tried to write line comment in a key"
        );
        debug_assert_ne!(
            self.elements.last(),
            Some(&ElementKind::Value),
            "tried to write line comment in a value"
        );

        self.write_indent(writer)?;
        writeln!(writer, "// {comment}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use std::error::Error;
    use std::io;

    #[inline]
    fn write_document<F, W, Fn>(f: &mut F, w: &mut W, fun: Fn) -> io::Result<()>
    where
        F: Formatter,
        W: ?Sized + Write,
        Fn: FnOnce(&mut F, &mut W) -> io::Result<()>,
    {
        f.begin_object(w)?;
        fun(f, w)?;
        f.end_object(w)
    }

    #[inline]
    fn write_obj<F, W, Fn>(f: &mut F, w: &mut W, fun: Fn) -> io::Result<()>
    where
        F: Formatter,
        W: ?Sized + Write,
        Fn: FnOnce(&mut F, &mut W) -> io::Result<()>,
    {
        f.begin_value(w)?;
        f.begin_object(w)?;
        fun(f, w)?;
        f.end_object(w)?;
        f.end_value(w)
    }

    #[inline]
    fn write_key<F, W>(f: &mut F, w: &mut W, key: &str) -> io::Result<()>
    where
        F: Formatter,
        W: ?Sized + Write,
    {
        f.begin_key(w)?;
        f.write_string(w, key)?;
        f.end_key(w)
    }

    #[inline]
    fn write_value<F, W>(f: &mut F, w: &mut W, v: &str) -> io::Result<()>
    where
        F: Formatter,
        W: ?Sized + Write,
    {
        f.begin_value(w)?;
        f.write_string(w, v)?;
        f.end_value(w)
    }

    fn write_simple_vmt<F, W>(f: &mut F, w: &mut W) -> io::Result<()>
    where
        F: Formatter,
        W: ?Sized + Write,
    {
        write_document(f, w, |f, w| {
            write_key(f, w, "LightmappedGeneric")?;
            write_obj(f, w, |f, w| {
                write_key(f, w, "$basetexture")?;
                write_value(f, w, "coast\\shingle_01")?;

                write_key(f, w, "$surfaceprop")?;
                write_value(f, w, "gravel")
            })
        })
    }

    fn write_nested_vdf<F, W>(f: &mut F, w: &mut W) -> io::Result<()>
    where
        F: Formatter,
        W: ?Sized + Write,
    {
        f.write_line_comment(w, "Test comment")?;
        write_document(f, w, |f, w| {
            write_key(f, w, "#base")?;
            write_value(f, w, "panelBase.res")?;

            write_key(f, w, "Resource/specificPanel.res")?;
            write_obj(f, w, |f, w| {
                write_key(f, w, "Greeting")?;
                write_value(f, w, "Hello, \"Bob\"!")?;

                write_key(f, w, "Nested")?;
                write_obj(f, w, |f, w| {
                    write_key(f, w, "Object")?;
                    write_value(f, w, "1")
                })
            })
        })
    }

    fn write_advanced_vdf<F, W>(f: &mut F, w: &mut W) -> io::Result<()>
    where
        F: Formatter,
        W: ?Sized + Write,
    {
        f.write_line_comment(w, "Auto-generated by VDFlex")?;
        write_document(f, w, |f, w| {
            write_key(f, w, "Basic Settings")?;
            write_obj(f, w, |f, w| {
                write_key(f, w, "Sound")?;
                write_obj(f, w, |f, w| {
                    write_key(f, w, "Volume")?;
                    write_value(f, w, "1.0")?;
                    write_key(f, w, "Enable voice")?;
                    write_value(f, w, "1")
                })?;
                write_key(f, w, "Controls")?;
                write_obj(f, w, |f, w| {
                    write_key(f, w, "Sensitivity")?;
                    write_value(f, w, "0.75")
                })
            })?;

            f.begin_key(w)?;
            f.write_string(w, "#include")?;
            f.write_conditional(w, "$WINDOWS")?;
            f.end_key(w)?;
            write_value(f, w, "sourcemods/{MODNAME}.vdf")?;

            f.begin_key(w)?;
            f.write_string(w, "#include")?;
            f.write_conditional(w, "$OSX")?;
            f.end_key(w)?;
            write_value(f, w, "sourcemods/{MODNAME}-macos.vdf")?;

            f.begin_key(w)?;
            f.write_string(w, "#include")?;
            f.write_conditional(w, "$LINUX")?;
            f.end_key(w)?;
            write_value(f, w, "sourcemods/{MODNAME}-linux.vdf")?;

            write_key(f, w, "Graphics")?;
            write_obj(f, w, |f, w| {
                f.write_line_comment(w, "needs to be a 3:4, 9:16 or 10:16 ratio")?;
                write_key(f, w, "Resolution")?;
                write_value(f, w, "[1920,1080]")
            })?;

            f.write_line_comment(w, "configure keybindings here")?;
            write_key(f, w, "Binds")?;
            write_obj(f, w, |f, w| {
                f.write_line_comment(w, "standard commands")?;

                write_key(f, w, "Bind")?;
                write_obj(f, w, |f, w| {
                    write_key(f, w, "key")?;
                    write_value(f, w, "w")?;
                    write_key(f, w, "command")?;
                    write_value(f, w, "+forward")
                })?;
                write_key(f, w, "Bind")?;
                write_obj(f, w, |f, w| {
                    write_key(f, w, "key")?;
                    write_value(f, w, "space")?;
                    write_key(f, w, "command")?;
                    write_value(f, w, "jump")
                })?;

                f.write_line_comment(w, "The most important command of all")?;
                write_key(f, w, "Bind")?;
                write_obj(f, w, |f, w| {
                    write_key(f, w, "key")?;
                    write_value(f, w, "p")?;
                    write_key(f, w, "command")?;
                    write_value(f, w, "say \"KABLOOIE\"; +explode")
                })
            })
        })
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

    #[test]
    fn advanced() -> Result<(), Box<dyn Error>> {
        let mut f = PrettyFormatter::with_opts(FormatOpts {
            indent: "    ".to_string(),
            separator: "  ".to_string(),
            brace_style: BraceStyle::Allman,
            quote_keys: Quoting::Always,
            quote_values: Quoting::WhenRequired,
            quote_macro_keys: Quoting::WhenRequired,
        });
        let mut buf = Vec::new();
        write_advanced_vdf(&mut f, &mut buf)?;
        assert_eq!(
            String::from_utf8(buf)?,
            indoc! {r##"
                // Auto-generated by VDFlex
                "Basic Settings"
                {
                    "Sound"
                    {
                        "Volume"  1.0
                        "Enable voice"  1
                    }
                    "Controls"
                    {
                        "Sensitivity"  0.75
                    }
                }
                #include [$WINDOWS]  "sourcemods/{MODNAME}.vdf"
                #include [$OSX]  "sourcemods/{MODNAME}-macos.vdf"
                #include [$LINUX]  "sourcemods/{MODNAME}-linux.vdf"
                "Graphics"
                {
                    // needs to be a 3:4, 9:16 or 10:16 ratio
                    "Resolution"  "[1920,1080]"
                }
                // configure keybindings here
                "Binds"
                {
                    // standard commands
                    "Bind"
                    {
                        "key"  w
                        "command"  +forward
                    }
                    "Bind"
                    {
                        "key"  space
                        "command"  jump
                    }
                    // The most important command of all
                    "Bind"
                    {
                        "key"  p
                        "command"  "say \"KABLOOIE\"; +explode"
                    }
                }
            "##}
        );
        Ok(())
    }

    #[test]
    fn advanced_compact() -> Result<(), Box<dyn Error>> {
        let mut f = PrettyFormatter::with_opts(FormatOpts {
            indent: "".to_string(),
            separator: " ".to_string(),
            brace_style: BraceStyle::KAndR,
            quote_keys: Quoting::WhenRequired,
            quote_values: Quoting::WhenRequired,
            quote_macro_keys: Quoting::WhenRequired,
        });
        let mut buf = Vec::new();
        write_advanced_vdf(&mut f, &mut buf)?;
        assert_eq!(
            String::from_utf8(buf)?,
            indoc! {r##"
                // Auto-generated by VDFlex
                "Basic Settings" {
                Sound {
                Volume 1.0
                "Enable voice" 1
                }
                Controls {
                Sensitivity 0.75
                }
                }
                #include [$WINDOWS] "sourcemods/{MODNAME}.vdf"
                #include [$OSX] "sourcemods/{MODNAME}-macos.vdf"
                #include [$LINUX] "sourcemods/{MODNAME}-linux.vdf"
                Graphics {
                // needs to be a 3:4, 9:16 or 10:16 ratio
                Resolution "[1920,1080]"
                }
                // configure keybindings here
                Binds {
                // standard commands
                Bind {
                key w
                command +forward
                }
                Bind {
                key space
                command jump
                }
                // The most important command of all
                Bind {
                key p
                command "say \"KABLOOIE\"; +explode"
                }
                }
            "##}
        );
        Ok(())
    }
}
