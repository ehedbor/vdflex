use indoc::indoc;
use serde::Serialize;
use std::f32::consts::PI;
use vdflex::ser::{
    to_string, to_string_flat, to_string_flat_pretty, to_string_nested_pretty, BraceStyle,
    FormatOpts, Quoting,
};
use vdflex::{Error, KeyValuesRoot, Result, RootKind};

#[derive(Serialize)]
struct UnitStruct;

#[derive(Serialize)]
struct NewTypeStruct(i32);

#[derive(Serialize)]
struct TupleStruct(i32, bool, String);

#[derive(Serialize)]
struct Struct {
    c: char,
    i: i32,
    s: String,
    b: bool,
}

#[derive(Serialize)]
enum Enum {
    UnitVariant,
    NewTypeVariant(char),
    TupleVariant(bool, String),
    StructVariant { c: char, i: i32 },
}

#[test]
fn serialize_root_level_primitives() -> Result<()> {
    let opts = FormatOpts {
        quote_values: Quoting::WhenRequired,
        ..Default::default()
    };

    // TODO: Is this really the right API? I didn't consider that you could serialize things that
    //     DON'T represent an entire KeyValues document.
    //     ...It's a little strange that you can, actually.
    assert_eq!(to_string_flat_pretty(&false, opts.clone())?, "0");
    assert_eq!(to_string_flat_pretty(&true, opts.clone())?, "1");
    assert_eq!(to_string_flat_pretty(&17u8, opts.clone())?, "17");
    assert_eq!(to_string_flat_pretty(&362i16, opts.clone())?, "362");
    assert_eq!(to_string_flat_pretty(&-843217i32, opts.clone())?, "-843217");
    assert_eq!(to_string_flat_pretty(&PI, opts.clone())?, PI.to_string());
    assert_eq!(
        to_string_flat_pretty(&u64::MAX, opts.clone())?,
        "18446744073709551615"
    );
    assert_eq!(to_string_flat_pretty(&'q', opts.clone())?, "q");
    assert_eq!(to_string_flat_pretty(&'\t', opts.clone())?, r#""\t""#);
    assert_eq!(to_string_flat_pretty(&"simple", opts.clone())?, "simple");
    assert_eq!(
        to_string_flat_pretty(&"Hello, world!", opts.clone())?,
        "\"Hello, world!\""
    );

    Ok(())
}

#[test]
fn serialize_option() -> Result<()> {
    let opts = FormatOpts {
        quote_values: Quoting::WhenRequired,
        ..Default::default()
    };

    assert_eq!(to_string_flat_pretty(&None::<i32>, opts.clone())?, "");
    assert_eq!(to_string_flat_pretty(&Some(42), opts.clone())?, "42");
    assert_eq!(
        to_string_flat_pretty(&Some("hello"), opts.clone())?,
        "hello"
    );

    Ok(())
}

#[test]
fn serialize_unit() -> Result<()> {
    let opts = FormatOpts {
        brace_style: BraceStyle::KAndR,
        quote_keys: Quoting::WhenRequired,
        quote_values: Quoting::WhenRequired,
        ..Default::default()
    };

    assert_eq!(to_string_flat_pretty(&(), opts.clone())?, "\"\"");
    assert_eq!(
        to_string_nested_pretty("Unit", &(), opts.clone())?,
        indoc! {r#"
            Unit ""
        "#}
    );

    assert_eq!(to_string_flat_pretty(&UnitStruct, opts.clone())?, "\"\"");
    assert_eq!(
        to_string_nested_pretty("Unit", &UnitStruct, opts.clone())?,
        indoc! {r#"
            Unit ""
        "#}
    );

    Ok(())
}

#[test]
fn serialize_new_type() -> Result<()> {
    let opts = FormatOpts {
        brace_style: BraceStyle::KAndR,
        quote_keys: Quoting::WhenRequired,
        quote_values: Quoting::WhenRequired,
        ..Default::default()
    };

    assert_eq!(to_string_flat_pretty(&NewTypeStruct(100), opts.clone())?, "100");
    assert_eq!(
        to_string_nested_pretty("NewTypeStruct", &(), opts.clone())?,
        indoc! {r#"
            NewTypeStruct 100
        "#}
    );

    assert_eq!(to_string_flat_pretty(&UnitStruct, opts.clone())?, "\"\"");
    assert_eq!(
        to_string_nested_pretty("Unit", &UnitStruct, opts.clone())?,
        indoc! {r#"
            Unit ""
        "#}
    );

    Ok(())
}

// TODO: Test the rest of the types
