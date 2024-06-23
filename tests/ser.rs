use indoc::indoc;
use serde::Serialize;
use std::f32::consts::PI;
use vdflex::ser::{
    kv_to_string, kv_to_string_pretty, to_string, to_string_pretty, BraceStyle, FormatOpts,
    PrettyFormatter, Quoting,
};
use vdflex::{Error, Result};

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

    assert_eq!(
        to_string_pretty(&false, PrettyFormatter::new(opts.clone()))?,
        "0"
    );
    assert_eq!(
        to_string_pretty(&true, PrettyFormatter::new(opts.clone()))?,
        "1"
    );
    assert_eq!(
        to_string_pretty(&17u8, PrettyFormatter::new(opts.clone()))?,
        "17"
    );
    assert_eq!(
        to_string_pretty(&362i16, PrettyFormatter::new(opts.clone()))?,
        "362"
    );
    assert_eq!(
        to_string_pretty(&-843217i32, PrettyFormatter::new(opts.clone()))?,
        "-843217"
    );
    assert_eq!(
        to_string_pretty(&PI, PrettyFormatter::new(opts.clone()))?,
        PI.to_string()
    );
    assert_eq!(
        to_string_pretty(&u64::MAX, PrettyFormatter::new(opts.clone()))?,
        "18446744073709551615"
    );
    assert_eq!(
        to_string_pretty(&'q', PrettyFormatter::new(opts.clone()))?,
        "q"
    );
    assert_eq!(
        to_string_pretty(&'\t', PrettyFormatter::new(opts.clone()))?,
        r#""\t""#
    );
    assert_eq!(
        to_string_pretty(&"simple", PrettyFormatter::new(opts.clone()))?,
        "simple"
    );
    assert_eq!(
        to_string_pretty(&"Hello, world!", PrettyFormatter::new(opts.clone()))?,
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

    assert_eq!(
        to_string_pretty(&None::<i32>, PrettyFormatter::new(opts.clone()))?,
        ""
    );
    assert_eq!(
        to_string_pretty(&Some(42), PrettyFormatter::new(opts.clone()))?,
        "42"
    );
    assert_eq!(
        to_string_pretty(&Some("hello"), PrettyFormatter::new(opts.clone()))?,
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

    assert_eq!(
        to_string_pretty(&(), PrettyFormatter::new(opts.clone()))?,
        "\"\""
    );
    assert_eq!(
        kv_to_string_pretty("Unit", &(), PrettyFormatter::new(opts.clone()))?,
        indoc! {r#"
            Unit ""
        "#}
    );

    assert_eq!(
        to_string_pretty(&UnitStruct, PrettyFormatter::new(opts.clone()))?,
        "\"\""
    );
    assert_eq!(
        kv_to_string_pretty("Unit", &UnitStruct, PrettyFormatter::new(opts.clone()))?,
        indoc! {r#"
            Unit ""
        "#}
    );

    Ok(())
}

#[test]
fn serialize_unit_struct() -> Result<()> {
    let opts = FormatOpts {
        quote_keys: Quoting::WhenRequired,
        quote_values: Quoting::WhenRequired,
        ..Default::default()
    };

    assert_eq!(
        to_string_pretty(&UnitStruct, PrettyFormatter::new(opts.clone()))?,
        "\"\""
    );
    assert_eq!(
        kv_to_string_pretty(
            "UnitStruct",
            &UnitStruct,
            PrettyFormatter::new(opts.clone())
        )?,
        indoc! {r#"
            UnitStruct ""
        "#}
    );

    Ok(())
}

#[test]
fn serialize_new_type_struct() -> Result<()> {
    let opts = FormatOpts {
        quote_keys: Quoting::WhenRequired,
        quote_values: Quoting::WhenRequired,
        ..Default::default()
    };

    assert_eq!(
        to_string_pretty(&NewTypeStruct(100), PrettyFormatter::new(opts.clone()))?,
        "100"
    );
    assert_eq!(
        kv_to_string_pretty(
            "NewTypeStruct",
            &NewTypeStruct(100),
            PrettyFormatter::new(opts.clone())
        )?,
        indoc! {r#"
            NewTypeStruct 100
        "#}
    );

    Ok(())
}

#[test]
fn serialize_tuple() -> Result<()> {
    let tuple = ("foo", 123, false, 'c', None::<i32>);

    assert!(matches!(to_string(&tuple), Err(Error::RootLevelSequence)));
    assert_eq!(
        kv_to_string("value", &tuple)?,
        indoc! {r#"
            "value" "foo"
            "value" "123"
            "value" "0"
            "value" "c"
            "value" ""
        "#}
    );

    let tuple = ((), 1, (2,), ((3,),), (((4,),),), ((((5),),),));
    assert!(matches!(to_string(&tuple), Err(Error::RootLevelSequence)));
    assert_eq!(
        kv_to_string_pretty(
            "element",
            &tuple,
            PrettyFormatter::new(FormatOpts {
                quote_keys: Quoting::WhenRequired,
                quote_values: Quoting::WhenRequired,
                ..Default::default()
            }),
        )?,
        indoc! {r#"
            element ""
            element 1
            element 2
            element 3
            element 4
            element 5
        "#}
    );

    Ok(())
}

#[test]
fn serialize_tuple_struct() -> Result<()> {
    let tuple_struct = TupleStruct(-36, true, String::from("{\"embeddedJson\":\"cursed\"}"));

    assert!(matches!(
        to_string(&tuple_struct),
        Err(Error::RootLevelSequence)
    ));
    assert_eq!(
        kv_to_string_pretty(
            "value",
            &tuple_struct,
            PrettyFormatter::new(FormatOpts {
                quote_keys: Quoting::WhenRequired,
                quote_values: Quoting::WhenRequired,
                ..Default::default()
            })
        )?,
        indoc! {r#"
            value -36
            value 1
            value "{\"embeddedJson\":\"cursed\"}"
        "#}
    );

    Ok(())
}

// TODO: Test the rest of the types
