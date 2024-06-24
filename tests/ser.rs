use std::collections::HashMap;
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
    NewTypeVariant(String),
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

#[test]
fn serialize_struct() -> Result<()> {
    let s = Struct {
        c: 'X',
        i: -123,
        s: String::from("Test data"),
        b: true,
    };

    assert_eq!(
        to_string(&s)?,
        indoc! {r#"
        "c" "X"
        "i" "-123"
        "s" "Test data"
        "b" "1"
    "#}
    );
    assert_eq!(
        kv_to_string("data", &s)?,
        indoc! {r#"
        "data"
        {
            "c" "X"
            "i" "-123"
            "s" "Test data"
            "b" "1"
        }
    "#}
    );

    Ok(())
}

#[test]
fn serialize_unit_variant() -> Result<()> {
    assert_eq!(
        kv_to_string("Variant", &Enum::UnitVariant)?,
        indoc! {r#"
            "Variant" "UnitVariant"
        "#},
    );
    Ok(())
}

#[test]
fn serialize_new_type_variant() -> Result<()> {
    assert_eq!(
        kv_to_string("Variant", &Enum::NewTypeVariant(String::from("inner")))?,
        indoc! {r#"
            "Variant"
            {
                "NewTypeVariant" "inner"
            }
        "#},
    );
    Ok(())
}

#[test]
fn serialize_tuple_variant() -> Result<()> {
    assert_eq!(
        kv_to_string("Variant", &Enum::TupleVariant(false, String::from("data")))?,
        indoc! {r#"
            "Variant"
            {
                "TupleVariant" "0"
                "TupleVariant" "data"
            }
        "#},
    );
    Ok(())
}

#[test]
fn serialize_struct_variant() -> Result<()> {
    assert_eq!(
        kv_to_string(
            "Variant",
            &Enum::StructVariant {
                c: 'K',
                i: 1_000_000
            }
        )?,
        indoc! {r#"
            "Variant"
            {
                "StructVariant"
                {
                    "c" "K"
                    "i" "1000000"
                }
            }
        "#},
    );
    Ok(())
}

#[test]
fn serialize_empty_collections() -> Result<()> {
    let vec = Vec::<()>::new();
    assert!(matches!(to_string(&vec), Err(Error::RootLevelSequence)));
    assert_eq!(kv_to_string("empty", &vec)?, "");
    
    let map = HashMap::<String, ()>::new();
    assert_eq!(to_string(&map)?, "");
    assert_eq!(
        kv_to_string("empty", &map)?,
        indoc! {r#"
            "empty"
            {
            }
        "#},
    );
    
    Ok(())
}

#[test]
fn serialize_nested_sequence() {
    let nested = vec![vec![10]];
    assert!(matches!(kv_to_string("nested", &nested), Err(Error::NestedSequence)));
    
    let very_nested = vec![vec![vec![vec![()]]]];
    assert!(matches!(kv_to_string("very_nested", &very_nested), Err(Error::NestedSequence)));
}

#[test]
fn serialize_sequence() -> Result<()> {
    let nums = vec![1.0, 2.0, 3.0];
    assert_eq!(
        kv_to_string("nums", &nums)?,
        indoc! {r#"
            "nums" "1.0"
            "nums" "2.0"
            "nums" "3.0"
        "#},
    );

    let variants = vec![
        Enum::UnitVariant,
        Enum::NewTypeVariant(String::from("Hello")),
        Enum::TupleVariant(true, String::from("Greetings, traveler.")),
        Enum::StructVariant { c: 'y', i: 2000 },
    ];

    assert_eq!(
        kv_to_string_pretty(
            "variants",
            &variants,
            PrettyFormatter::new(FormatOpts {
                brace_style: BraceStyle::KAndR,
                quote_keys: Quoting::WhenRequired,
                quote_values: Quoting::WhenRequired,
                ..Default::default()
            })
        )?,
        indoc! {r#"
            variants UnitVariant
            variants {
                NewTypeVariant Hello
            }
            variants {
                TupleVariant 1
                TupleVariant "Greetings, traveler."
            }
            variants {
                StructVariant {
                    c y
                    i 2000
                }
            }
        "#},
    );

    Ok(())
}

#[test]
fn serialize_map() {
    todo!()
}
