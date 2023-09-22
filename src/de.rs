use crate::{Error, Result};
use serde::de::DeserializeOwned;
use serde::Deserialize;

pub fn from_str<'a, T>(s: &'a str) -> Result<T>
where
    T: Deserialize<'a>,
{
    todo!()
}

#[cfg(feature = "std")]
pub fn from_reader<R, T>(reader: R) -> Result<T>
where
    R: std::io::Read,
    T: DeserializeOwned,
{
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Keyvalues, Value};
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
    fn test_de_simple_keyvalues() {
        let vdf: Keyvalues = from_str(SIMPLE_KEYVALUES).unwrap();

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
    fn test_de_simple_struct() {
        let foo: Foo = from_str(SIMPLE_KEYVALUES).unwrap();
        assert_eq!(foo.bar, "baz");
    }
}
