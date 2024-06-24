use super::formatter::{Formatter, PrettyFormatter};
use crate::{Error, Result};
use serde::ser::{
    Impossible, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
    SerializeTuple, SerializeTupleStruct, SerializeTupleVariant,
};
use serde::Serialize;
use std::borrow::Cow;
use std::io::Write;

/// Serializes Rust types into KeyValues text.
pub struct Serializer<W, F = PrettyFormatter> {
    writer: W,
    formatter: F,
    elements: Vec<Option<Cow<'static, str>>>,
}

impl<W: Write, F: Formatter> Serializer<W, F> {
    /// Creates a new KeyValues serializer using the given `writer` and `formatter`.
    pub fn new(writer: W, formatter: F) -> Self {
        Self {
            writer,
            formatter,
            elements: Vec::new(),
        }
    }

    fn begin_seq(&mut self) -> Result<()> {
        // Make sure sequences are enclosed in maps
        match self.elements.last() {
            Some(Some(_)) => Ok(()),
            _ => Err(Error::UnrepresentableSequence),
        }
    }
    
    fn end_seq(&mut self) -> Result<()> {
        Ok(())
    }
    
    fn begin_map(&mut self) -> Result<()> {
        if let Some(key) = Self::current_key(&self.elements) {
            self.formatter
                .begin_key(&mut self.writer)
                .and_then(|_| self.formatter.write_string(&mut self.writer, key))
                .and_then(|_| self.formatter.end_key(&mut self.writer))
                .and_then(|_| self.formatter.begin_value(&mut self.writer))
                .map_err(|e| Error::Io(e))?;
        }

        self.formatter
            .begin_object(&mut self.writer)
            .map_err(|e| Error::Io(e))
    }

    fn end_map(&mut self) -> Result<()> {
        self.formatter
            .end_object(&mut self.writer)
            .map_err(|e| Error::Io(e))?;
            
        if !self.elements.is_empty() {
            self.formatter
                .end_value(&mut self.writer)
                .map_err(|e| Error::Io(e))?;
        }
        
        Ok(())
    }

    /// Begins a map element (when `key` is `Some`) or sequence element (when `key` is `None).
    fn begin_element(&mut self, key: Option<Cow<'static, str>>) -> Result<()> {
        self.elements.push(key);
        Ok(())
    }

    /// Ends the current map or sequence element.
    fn end_element(&mut self) -> Result<()> {
        self.elements.pop();
        Ok(())
    }

    fn string_value(&mut self, value: &str) -> Result<()> {
        if let Some(key) = Self::current_key(&self.elements) {
            // We're in a map or sequence. Write a key-value.
            self.formatter
                .begin_key(&mut self.writer)
                .and_then(|_| self.formatter.write_string(&mut self.writer, key))
                .and_then(|_| self.formatter.end_key(&mut self.writer))
                .and_then(|_| self.formatter.begin_value(&mut self.writer))
                .and_then(|_| self.formatter.write_string(&mut self.writer, value))
                .and_then(|_| self.formatter.end_value(&mut self.writer))
                .map_err(|e| Error::Io(e))
        } else {
            // We're at the root level. Just write the plain string.
            self.formatter
                .write_string(&mut self.writer, value)
                .map_err(|e| Error::Io(e))
        }
    }
    
    fn current_key<'a>(elements: &'a Vec<Option<Cow<'a, str>>>) -> Option<&'a str> {
        elements.last().map(|element| match element {
            Some(direct_key) => direct_key.as_ref(),
            None => elements
                .iter()
                .nth_back(1)
                .expect("found root-level list? (should be impossible)")
                .as_ref()
                .expect("found nested list? (should be impossible)"),
        })
    }
}

macro_rules! serialize_as_str_impl {
    ($ty:ident) => {
        paste::paste! {
            fn [<serialize_ $ty>](self, v: $ty) -> $crate::Result<Self::Ok> {
                self.serialize_str(&v.to_string())
            }
        }
    };
    ($first:ident, $($rest:ident),+) => {
        serialize_as_str_impl!($first);
        serialize_as_str_impl!($($rest),+);
    }
}

impl<'a, W: Write, F: Formatter> serde::Serializer for &'a mut Serializer<W, F> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        let s = if v { "1" } else { "0" };
        self.serialize_str(s)
    }

    serialize_as_str_impl!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, f32, f64, char);
    
    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        self.string_value(v)
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok> {
        Err(Error::UnsupportedType("bytes".to_string()))
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        // omit the value entirely, unless we're at root level.
        if self.elements.is_empty() {
            self.string_value("")?;
        }
        
        Ok(())
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        self.serialize_none()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok> {
        self.begin_map()?;
        self.begin_element(Some(Cow::Borrowed(variant)))?;
        value.serialize(&mut *self)?;
        self.end_element()?;
        self.end_map()?;

        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.begin_seq()?;
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.begin_map()?;
        self.begin_element(Some(Cow::Borrowed(variant)))?;
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        self.begin_map()?;
        Ok(self)
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        self.begin_map()?;
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.begin_map()?;
        self.begin_element(Some(Cow::Borrowed(variant)))?;
        self.begin_map()?;
        Ok(self)
    }
}

struct MapKeySerializer<'a, W, F> {
    serializer: &'a mut Serializer<W, F>,
}

impl<'a, W: Write, F: Formatter> serde::Serializer for MapKeySerializer<'a, W, F> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    serialize_as_str_impl!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, f32, f64, char);

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        self.serialize_str(if v { "1" } else { "0" })
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        let key = Cow::Owned(String::from(v));
        self.serializer.begin_element(Some(key))
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok> {
        Err(Error::KeyMustBeAString("bytes".to_string()))
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        Err(Error::KeyMustBeAString("none".to_string()))
    }

    fn serialize_some<T: ?Sized + Serialize>(self, _value: &T) -> Result<Self::Ok> {
        Err(Error::KeyMustBeAString("some".to_string()))
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Err(Error::KeyMustBeAString("unit".to_string()))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        Err(Error::KeyMustBeAString("unit struct".to_string()))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok> {
        Err(Error::KeyMustBeAString("unit variant".to_string()))
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok> {
        Err(Error::KeyMustBeAString("newtype variant".to_string()))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(Error::KeyMustBeAString("sequence".to_string()))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(Error::KeyMustBeAString("tuple".to_string()))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(Error::KeyMustBeAString("tuple struct".to_string()))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::KeyMustBeAString("tuple variant".to_string()))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::KeyMustBeAString("map".to_string()))
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(Error::KeyMustBeAString("struct".to_string()))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::KeyMustBeAString("struct variant".to_string()))
    }
}

impl<'a, W: Write, F: Formatter> SerializeSeq for &'a mut Serializer<W, F> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<Self::Ok> {
        self.begin_element(None)?;
        value.serialize(&mut **self)?;
        self.end_element()
    }

    fn end(self) -> Result<Self::Ok> {
        self.end_seq()
    }
}

impl<'a, W: Write, F: Formatter> SerializeTuple for &'a mut Serializer<W, F> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<Self::Ok> {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        SerializeSeq::end(self)
    }
}

impl<'a, W: Write, F: Formatter> SerializeTupleStruct for &'a mut Serializer<W, F> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<Self::Ok> {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        SerializeSeq::end(self)
    }
}

impl<'a, W: Write, F: Formatter> SerializeTupleVariant for &'a mut Serializer<W, F> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<Self::Ok> {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok> {
        self.end_element()?;
        self.end_map()
    }
}

impl<'a, W: Write, F: Formatter> SerializeMap for &'a mut Serializer<W, F> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<Self::Ok> {
        let ser = MapKeySerializer { serializer: self };
        key.serialize(ser)
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<Self::Ok> {
        value.serialize(&mut **self)?;
        self.end_element()
    }

    fn end(self) -> Result<Self::Ok> {
        self.end_map()
    }
}

impl<'a, W: Write, F: Formatter> SerializeStruct for &'a mut Serializer<W, F> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<Self::Ok> {
        self.begin_element(Some(Cow::Borrowed(key)))?;
        value.serialize(&mut **self)?;
        self.end_element()
    }

    fn end(self) -> Result<Self::Ok> {
        self.end_map()
    }
}

impl<'a, W: Write, F: Formatter> SerializeStructVariant for &'a mut Serializer<W, F> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<Self::Ok> {
        SerializeStruct::serialize_field(self, key, value)
    }

    fn end(self) -> Result<Self::Ok> {
        self.end_map()?;
        self.end_element()?;
        self.end_map()
    }
}
