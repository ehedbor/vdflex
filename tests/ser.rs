use indoc::indoc;
use serde::Serialize;
use std::collections::HashMap;
use std::f32::consts::PI;
use vdflex::ser::{
    kv_to_string, kv_to_string_pretty, to_string, to_string_pretty, BraceStyle, FormatOpts,
    PrettyFormatter, Quoting,
};
use vdflex::{Error, KeyValues, Object, Result, Value};

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
        "\"\""
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
    assert_eq!(to_string(&())?, "\"\"");
    assert_eq!(kv_to_string("Unit", &())?, "");

    Ok(())
}

#[test]
fn serialize_unit_struct() -> Result<()> {
    assert_eq!(to_string(&UnitStruct)?, "\"\"");
    assert_eq!(kv_to_string("Unit", &UnitStruct)?, "");

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

    assert!(matches!(
        to_string(&tuple),
        Err(Error::UnrepresentableSequence)
    ));
    assert_eq!(
        kv_to_string("value", &tuple)?,
        indoc! {r#"
            "value" "foo"
            "value" "123"
            "value" "0"
            "value" "c"
        "#}
    );

    let tuple = ((), 1, (2,), ((3,),), (((4,),),), ((((5,),),),));
    assert!(matches!(
        to_string(&tuple),
        Err(Error::UnrepresentableSequence)
    ));
    assert!(matches!(
        kv_to_string("element", &tuple),
        Err(Error::UnrepresentableSequence)
    ));

    Ok(())
}

#[test]
fn serialize_tuple_struct() -> Result<()> {
    let tuple_struct = TupleStruct(-36, true, String::from("{\"embeddedJson\":\"cursed\"}"));

    assert!(matches!(
        to_string(&tuple_struct),
        Err(Error::UnrepresentableSequence)
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
    assert!(matches!(
        to_string(&vec),
        Err(Error::UnrepresentableSequence)
    ));
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
    assert!(matches!(
        kv_to_string("nested", &nested),
        Err(Error::UnrepresentableSequence)
    ));

    let very_nested = vec![vec![vec![vec![()]]]];
    assert!(matches!(
        kv_to_string("very_nested", &very_nested),
        Err(Error::UnrepresentableSequence)
    ));

    let mut tricky = HashMap::new();
    tricky.insert("this won't fool me!", vec![vec!["or will it?"]]);
    assert!(matches!(
        kv_to_string("tricky", &tricky),
        Err(Error::UnrepresentableSequence)
    ));
}

#[test]
fn serialize_sequence() -> Result<()> {
    let nums = vec![1.0, 2.0, 3.0];
    assert_eq!(
        kv_to_string("nums", &nums)?,
        indoc! {r#"
            "nums" "1"
            "nums" "2"
            "nums" "3"
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
#[cfg(feature = "indexmap")]
fn serialize_map() -> Result<()> {
    let mut properties = indexmap::IndexMap::new();
    properties.insert("Buzzword", vec!["foo", "bar", "baz"]);
    properties.insert("Flag", vec!["1"]);
    properties.insert(
        "Comment",
        vec!["I was planning on testing nested maps here but the type system stopped me >:("],
    );

    assert_eq!(
        kv_to_string_pretty(
            "Properties",
            &properties,
            PrettyFormatter::new(FormatOpts {
                quote_keys: Quoting::WhenRequired,
                quote_values: Quoting::WhenRequired,
                ..Default::default()
            })
        )?,
        indoc! {r#"
            Properties
            {
                Buzzword foo
                Buzzword bar
                Buzzword baz
                Flag 1
                Comment "I was planning on testing nested maps here but the type system stopped me >:("
            }
        "#},
    );

    Ok(())
}

#[test]
#[cfg(feature = "preserve_order")]
fn serialize_key_values() -> Result<()> {
    fn set_str(obj: &mut Object, key: impl ToString, value: impl ToString) {
        obj.insert(key.to_string(), vec![Value::String(value.to_string())]);
    }

    let mut properties = Object::new();
    set_str(&mut properties, "$basetexture", "water/water_still");
    set_str(&mut properties, "$surfaceprop", "water");
    set_str(&mut properties, "$transluscent", "1");
    set_str(&mut properties, "%compilewater", "1");
    set_str(&mut properties, "%tooltexture", "water/water_still_frame00");
    set_str(&mut properties, "$abovewater", "1");
    set_str(
        &mut properties,
        "$bottommaterial",
        "water/water_still_beneath",
    );
    set_str(&mut properties, "$fogenable", "1");
    set_str(&mut properties, "$fogcolor", "{5 5 51}");
    set_str(&mut properties, "$fogstart", "0");
    set_str(&mut properties, "$fogend", "200");
    set_str(&mut properties, "$lightmapwaterfog", "1");
    set_str(&mut properties, "$flashlightttint", "1");

    let mut animated_texture = Object::new();
    animated_texture.insert(
        String::from("animatedTextureVar"),
        vec![Value::String(String::from("$basetexture"))],
    );
    animated_texture.insert(
        String::from("animatedTextureFrameNumVar"),
        vec![Value::String(String::from("$frame"))],
    );
    animated_texture.insert(
        String::from("animatedTextureFrameRate"),
        vec![Value::String(String::from("10"))],
    );

    let mut proxies = Object::new();
    proxies.insert(
        String::from("AnimatedTexture"),
        vec![Value::Object(animated_texture)],
    );

    properties.insert(String::from("Proxies"), vec![Value::Object(proxies)]);

    let vmt = KeyValues::new(
        String::from("LightmappedGeneric"),
        Value::Object(properties),
    );

    assert_eq!(
        to_string_pretty(
            &vmt,
            PrettyFormatter::new(FormatOpts {
                quote_keys: Quoting::WhenRequired,
                quote_values: Quoting::WhenRequired,
                ..Default::default()
            })
        )?,
        indoc! {r#"
            LightmappedGeneric
            {
                $basetexture water/water_still
                $surfaceprop water
                $transluscent 1
                %compilewater 1
                %tooltexture water/water_still_frame00
                $abovewater 1
                $bottommaterial water/water_still_beneath
                $fogenable 1
                $fogcolor "{5 5 51}"
                $fogstart 0
                $fogend 200
                $lightmapwaterfog 1
                $flashlightttint 1
                Proxies
                {
                    AnimatedTexture
                    {
                        animatedTextureVar $basetexture
                        animatedTextureFrameNumVar $frame
                        animatedTextureFrameRate 10
                    }
                }
            }
        "#},
    );

    Ok(())
}
