use crate::{Error, KeyValuesRoot, Result, RootKind};
use serde::de::DeserializeOwned;
use serde::Deserialize;

/// Deserialize KeyValues text to some type `T`.
///
/// # Errors
///
/// Deserialization can fail if the input is not valid KeyValues or does not match the structure
/// expected by `T`. It can also fail if `T`'s implementation of `Deserialize` decides to fail.
pub fn from_str<'a, T>(s: &'a str) -> Result<T>
where
    T: Deserialize<'a> + KeyValuesRoot,
{
    match T::kind() {
        RootKind::Nested(root_key) => {
            let (key, value) = from_str_nested(s)?;
            if key != root_key {
                Err(Error::UnsupportedKey(key))
            } else {
                Ok(value)
            }
        }
        RootKind::Flattened => from_str_flat(s),
    }
}

/// Deserialize KeyValues text representing a single key-value pair to the key and some type `T`.
///
/// # Errors
///
/// Deserialization can fail if the input is not valid KeyValues or does not match the structure
/// expected by `T`. It can also fail if `T`'s implementation of `Deserialize` decides to fail.
pub fn from_str_nested<'a, T>(_s: &'a str) -> Result<(String, T)>
where
    T: Deserialize<'a>,
{
    todo!()
}

/// Deserialize KeyValues text representing a flattened object to some type `T`.
///
/// # Errors
///
/// Deserialization can fail if the input is not valid KeyValues or does not match the structure
/// expected by `T`. It can also fail if `T`'s implementation of `Deserialize` decides to fail.
pub fn from_str_flat<'a, T>(_s: &'a str) -> Result<T>
where
    T: Deserialize<'a>,
{
    todo!()
}

/// Deserialize KeyValues text from a reader to some type `T`.
///
/// # Errors
///
/// Deserialization can fail if the input is not valid KeyValues or does not match the structure
/// expected by `T`. It can also fail if `T`'s implementation of `Deserialize` decides to fail.
#[cfg(feature = "std")]
pub fn from_reader<R, T>(reader: R) -> Result<(String, T)>
where
    R: std::io::Read,
    T: DeserializeOwned + KeyValuesRoot,
{
    match T::kind() {
        RootKind::Nested(root_key) => {
            let (key, value) = from_reader_nested(reader)?;
            if key != root_key {
                Err(Error::UnsupportedKey(key))
            } else {
                Ok(value)
            }
        }
        RootKind::Flattened => from_reader_flat(reader),
    }
}

/// Deserialize KeyValues text representing a single key-value pair from a reader to the key and
/// some type `T`.
///
/// # Errors
///
/// Deserialization can fail if the input is not valid KeyValues or does not match the structure
/// expected by `T`. It can also fail if `T`'s implementation of `Deserialize` decides to fail.
#[cfg(feature = "std")]
pub fn from_reader_nested<R, T>(_reader: R) -> Result<(String, T)>
where
    R: std::io::Read,
    T: DeserializeOwned,
{
    todo!()
}

/// Deserialize KeyValues text representing a flattened object from a reader to some type `T`.
///
/// # Errors
///
/// Deserialization can fail if the input is not valid KeyValues or does not match the structure
/// expected by `T`. It can also fail if `T`'s implementation of `Deserialize` decides to fail.
#[cfg(feature = "std")]
pub fn from_reader_flat<R, T>(_reader: R) -> Result<T>
where
    R: std::io::Read,
    T: DeserializeOwned,
{
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{KeyValues, Value};
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
    fn de_simple_key_values() {
        let vdf: KeyValues = from_str_flat(SIMPLE_KEYVALUES).unwrap();

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
    fn de_simple_struct() {
        let (key, foo) = from_str_nested::<Foo>(SIMPLE_KEYVALUES).unwrap();
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

    impl KeyValuesRoot for Animals {
        fn kind() -> RootKind {
            RootKind::Flattened
        }
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
    fn de_animals() -> Result<()> {
        let animals = from_str_nested::<Animals>(ANIMALS);
        assert!(matches!(animals, Err(Error::MultipleRootKeys)));

        let animals: Animals = from_str(ANIMALS)?;
        let animals2: Animals = from_str_flat(ANIMALS)?;
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
