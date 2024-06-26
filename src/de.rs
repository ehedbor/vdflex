//! Deserialize KeyValues text to Rust types.

use crate::Result;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::io::Read;

/// Deserialize a KeyValues value representing some type `T`.
///
/// # Errors
///
/// Deserialization can fail if the input is not valid KeyValues or does not match the structure
/// expected by `T`. It can also fail if `T`'s implementation of `Deserialize` decides to fail.
pub fn from_str<'a, T: Deserialize<'a>>(_s: &'a str) -> Result<T> {
    unimplemented!()
}

/// Deserialize a KeyValues object representing a single key-value pair mapping a string key to
/// some type `T`.
///
/// # Errors
///
/// Deserialization can fail if the input is not valid KeyValues or does not match the structure
/// expected by `T`. It can also fail if `T`'s implementation of `Deserialize` decides to fail.
pub fn kv_from_str<'a, T: Deserialize<'a>>(_s: &'a str) -> Result<(String, T)> {
    unimplemented!()
}

/// Deserialize a KeyValues value representing some type `T` from a reader.
///
/// # Errors
///
/// Deserialization can fail if the input is not valid KeyValues or does not match the structure
/// expected by `T`. It can also fail if `T`'s implementation of `Deserialize` decides to fail.
#[cfg(feature = "std")]
pub fn from_reader<R: Read, T: DeserializeOwned>(_reader: R) -> Result<T> {
    unimplemented!()
}

/// Deserialize a KeyValues object representing a single key-value pair mapping a string key to
/// some type `T`, from a reader.
///
///
/// # Errors
///
/// Deserialization can fail if the input is not valid KeyValues or does not match the structure
/// expected by `T`. It can also fail if `T`'s implementation of `Deserialize` decides to fail.
#[cfg(feature = "std")]
pub fn kv_from_reader<R: Read, T: DeserializeOwned>(_reader: R) -> Result<(String, T)> {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Error, KeyValues, Value};
    use indoc::indoc;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Foo {
        pub bar: String,
    }

    const SIMPLE_KEYVALUES: &'static str = indoc! {r##"
        // This is a comment. It should not be parsed. This is verified by
        // adding some bizzare comments.

        foo{//start an object with { and end it with } }
            bar   baz // define the property "bar" with the value "baz
        }// end an object with }
    "##};

    #[test]
    #[ignore]
    fn de_simple_key_values() {
        let vdf: KeyValues = from_str(SIMPLE_KEYVALUES).unwrap();

        assert_eq!(vdf.root.len(), 1);
        assert_eq!(vdf.root["foo"].len(), 1);
        let foo = match &vdf.root["foo"][0] {
            Value::String(_) => panic!("expected object"),
            Value::Object(obj) => obj,
        };

        assert_eq!(foo.len(), 1);
        assert_eq!(foo["bar"].len(), 1);
        let bar = match &foo["bar"][0] {
            Value::String(s) => s,
            Value::Object(_) => panic!("expected string"),
        };

        assert_eq!(bar, "baz");
    }

    #[test]
    #[ignore]
    fn de_simple_struct() {
        let (key, foo) = kv_from_str::<Foo>(SIMPLE_KEYVALUES).unwrap();
        assert_eq!(key, "foo");
        assert_eq!(foo.bar, "baz");
    }

    const ANIMALS: &'static str = indoc! {r##"
        "Cats" {
            "Cat" {
                "Name" "Archie"
                "Age" "2"
            }
            "Cat" {
                "Name" "Boots"
                "Age" "22"
                "LikesCatnip" "0"
            }
        }
        "Dogs" {
            "Dog" {
                "Name" "Teddy"
                "Age" "6"
                "IsGoodDog" "1"
            }
            "Dog" {
                "Name" "Lucy"
                "Age" "5"
                "IsGoodDog" "1"
            }
        }
    "##};

    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(rename = "CamelCase")]
    struct Animals {
        cats: Cats,
        dogs: Dogs,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct Cats {
        #[serde(rename = "Cat")]
        items: Vec<Cat>,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct Dogs {
        #[serde(rename = "Dog")]
        items: Vec<Dog>,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(rename = "CamelCase")]
    struct Cat {
        name: String,
        age: i32,
        likes_catnip: Option<bool>,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(rename = "CamelCase")]
    struct Dog {
        name: String,
        age: i32,
        is_good_dog: bool,
    }

    #[test]
    #[ignore]
    fn de_animals() -> Result<()> {
        let animals = kv_from_str::<Animals>(ANIMALS);
        assert!(matches!(animals, Err(Error::MultipleRootKeys)));

        let animals: Animals = from_str(ANIMALS)?;
        let animals2: Animals = from_str(ANIMALS)?;
        assert_eq!(animals, animals2);
        assert_eq!(
            animals,
            Animals {
                cats: Cats {
                    items: vec![
                        Cat {
                            name: String::from("Archie"),
                            age: 2,
                            likes_catnip: None
                        },
                        Cat {
                            name: String::from("Boots"),
                            age: 22,
                            likes_catnip: Some(false),
                        },
                    ]
                },
                dogs: Dogs {
                    items: vec![
                        Dog {
                            name: String::from("Teddy"),
                            age: 6,
                            is_good_dog: true
                        },
                        Dog {
                            name: String::from("Lucy"),
                            age: 5,
                            is_good_dog: true,
                        },
                    ]
                },
            }
        );

        Ok(())
    }
}
