use super::formatter::{Formatter, PrettyFormatter};
use crate::{Error, Result};
use serde::ser::Impossible;
use serde::Serialize;
use std::borrow::Cow;
use std::io::Write;

pub struct Serializer<W, F = PrettyFormatter> {
    writer: W,
    formatter: F,
    key_stack: Vec<Cow<'static, str>>,
}

impl<W: Write, F: Formatter> Serializer<W, F> {
    /// Creates a new KeyValues serializer using the given `writer` and `formatter`.
    pub fn new(writer: W, formatter: F) -> Self {
        Self {
            writer,
            formatter,
            key_stack: Vec::new(),
        }
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

    serialize_as_str_impl!(i8, i16, i32, i64, u8, u16, u32, u64, char);

    fn serialize_i128(self, _v: i128) -> Result<Self::Ok> {
        Err(Error::UnsupportedType("i128".to_string()))
    }

    fn serialize_u128(self, _v: u128) -> Result<Self::Ok> {
        Err(Error::UnsupportedType("u128".to_string()))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        if v.is_finite() {
            self.serialize_str(&v.to_string())
        } else {
            Err(Error::NonFiniteFloat(v as f64))
        }
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        // Technically, VDF doesn't support doubles... or at least, one particular implementation
        // doesn't support them. I assume that I won't break anything by trying to serialize as
        // accurate a float as possible.
        if v.is_finite() {
            self.serialize_str(&v.to_string())
        } else {
            Err(Error::NonFiniteFloat(v))
        }
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        self.formatter
            .write_string(&mut self.writer, v)
            .map_err(|e| Error::Io(e))
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok> {
        Err(Error::UnsupportedType("bytes".to_string()))
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        // TODO: this should be represented by omitting the key
        self.serialize_str("")
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
        self.formatter
            .begin_object(&mut self.writer)
            .map_err(|e| Error::Io(e))?;

        self.key_stack.push(Cow::Borrowed(variant));
        self.formatter
            .begin_key(&mut self.writer)
            .map_err(|e| Error::Io(e))?;
        self.formatter
            .write_string(&mut self.writer, variant)
            .map_err(|e| Error::Io(e))?;
        self.formatter
            .end_key(&mut self.writer)
            .map_err(|e| Error::Io(e))?;

        self.formatter
            .begin_value(&mut self.writer)
            .map_err(|e| Error::Io(e))?;
        value.serialize(&mut *self)?;
        self.formatter
            .end_value(&mut self.writer)
            .map_err(|e| Error::Io(e))?;
        self.key_stack.pop();

        self.formatter
            .end_object(&mut self.writer)
            .map_err(|e| Error::Io(e))?;

        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
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
        self.serialize_tuple(len)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.key_stack.push(Cow::Borrowed(variant));
        self.formatter
            .begin_object(&mut self.writer)
            .map_err(|e| Error::Io(e))?;
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        self.formatter
            .begin_object(&mut self.writer)
            .map_err(|e| Error::Io(e))?;
        Ok(self)
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        self.formatter
            .begin_object(&mut self.writer)
            .map_err(|e| Error::Io(e))?;
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.formatter
            .begin_object(&mut self.writer)
            .map_err(|e| Error::Io(e))?;

        self.key_stack.push(Cow::Borrowed(variant));
        self.formatter
            .begin_key(&mut self.writer)
            .map_err(|e| Error::Io(e))?;
        self.formatter
            .write_string(&mut self.writer, variant)
            .map_err(|e| Error::Io(e))?;
        self.formatter
            .end_key(&mut self.writer)
            .map_err(|e| Error::Io(e))?;

        self.formatter
            .begin_value(&mut self.writer)
            .map_err(|e| Error::Io(e))?;
        self.formatter
            .begin_object(&mut self.writer)
            .map_err(|e| Error::Io(e))?;
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

    serialize_as_str_impl!(i8, i16, i32, i64, u8, u16, u32, u64, char);

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        self.serialize_str(if v { "1" } else { "0" })
    }

    fn serialize_i128(self, _v: i128) -> Result<Self::Ok> {
        Err(Error::UnsupportedType("i128".to_string()))
    }

    fn serialize_u128(self, _v: u128) -> Result<Self::Ok> {
        Err(Error::UnsupportedType("u128".to_string()))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        if !v.is_finite() {
            Err(Error::NonFiniteFloat(v as f64))
        } else {
            self.serialize_str(&v.to_string())
        }
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        if !v.is_finite() {
            Err(Error::NonFiniteFloat(v))
        } else {
            self.serialize_str(&v.to_string())
        }
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        self.serializer.key_stack.push(Cow::Owned(String::from(v)));
        self.serializer
            .formatter
            .begin_key(&mut self.serializer.writer)
            .map_err(|e| Error::Io(e))?;
        v.serialize(&mut *self.serializer)?;
        self.serializer
            .formatter
            .end_key(&mut self.serializer.writer)
            .map_err(|e| Error::Io(e))
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok> {
        Err(Error::KeyMustBeAString("bytes".to_string()))
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        Err(Error::KeyMustBeAString("bytes".to_string()))
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok> {
        value.serialize(self)
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

impl<'a, W: Write, F: Formatter> serde::ser::SerializeSeq for &'a mut Serializer<W, F> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<Self::Ok> {
        let key = self.key_stack.last().ok_or(Error::RootLevelSequence)?;

        // TODO: Fix Map<String, Sequence<T>> serialization
        // TL;DR: Map<_, Sequence<_>> needs to be special-cased.
        //
        // Maps serialize keys and values separately, first writing the key, then the value. This is
        // problematic when serializing a map containing sequences as its value, because it is 
        // assumed here that we ONLY write the key(s) HERE! As a result, the first element's key is
        // written TWICE.
        //
        // As a fix, we COULD check if we've already begun a KeyValue by checking the formatter's 
        // element stack. If so, we simply skip the key for now. Future elements would write the key
        // as expected. 
        // 
        // Unfortunately, this doesn't work in practice. First of all, SerializeMap also calls 
        // begin_value and end_value so we're already screwed. Second, PrettyFormatter (the only 
        // Formatter impl as of yet) keeps track of which elements it's currently considering, but 
        // the generic Formatter trait does not have any such requirement. I also feel it would be 
        // strange to introduce a requirement to expose what is mostly intended as a sanity check. 
        // Third, this doesn't even handle empty sequences.
        //
        // See, remember where I said that SerializeMap always writes a key before writing a value?
        // This also happens for empty sequences! Empty sequences shouldn't print anything at all.
        // We can't just "delete" the key once we realize we don't need it, either. Once the key is
        // written, it's written and we can't do anything about it.
        //
        // Clearly, a more involved solution is necessary. We need to be able to remember that we
        // might need to write a key and only commit it once we realize we do, in fact, need it.
        
        self.formatter
            .begin_key(&mut self.writer)
            .map_err(|e| Error::Io(e))?;
        self.formatter
            .write_string(&mut self.writer, key)
            .map_err(|e| Error::Io(e))?;
        self.formatter
            .end_key(&mut self.writer)
            .map_err(|e| Error::Io(e))?;

        self.formatter
            .begin_value(&mut self.writer)
            .map_err(|e| Error::Io(e))?;
        value.serialize(&mut **self)?;
        self.formatter
            .end_value(&mut self.writer)
            .map_err(|e| Error::Io(e))?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a, W: Write, F: Formatter> serde::ser::SerializeTuple for &'a mut Serializer<W, F> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<Self::Ok> {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl<'a, W: Write, F: Formatter> serde::ser::SerializeTupleStruct for &'a mut Serializer<W, F> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<Self::Ok> {
        serde::ser::SerializeTuple::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        serde::ser::SerializeTuple::end(self)
    }
}

impl<'a, W: Write, F: Formatter> serde::ser::SerializeTupleVariant for &'a mut Serializer<W, F> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<Self::Ok> {
        serde::ser::SerializeTuple::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        self.formatter
            .end_object(&mut self.writer)
            .map_err(|e| Error::Io(e))?;
        self.key_stack.pop();
        Ok(())
    }
}

impl<'a, W: Write, F: Formatter> serde::ser::SerializeMap for &'a mut Serializer<W, F> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<Self::Ok> {
        let ser = MapKeySerializer { serializer: self };
        key.serialize(ser)
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<Self::Ok> {
        self.formatter
            .begin_value(&mut self.writer)
            .map_err(|e| Error::Io(e))?;
        value.serialize(&mut **self)?;
        self.formatter
            .end_value(&mut self.writer)
            .map_err(|e| Error::Io(e))?;
        self.key_stack.pop();
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        self.formatter
            .end_object(&mut self.writer)
            .map_err(|e| Error::Io(e))
    }
}

impl<'a, W: Write, F: Formatter> serde::ser::SerializeStruct for &'a mut Serializer<W, F> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<Self::Ok> {
        self.key_stack.push(Cow::Borrowed(key));

        self.formatter
            .begin_key(&mut self.writer)
            .map_err(|e| Error::Io(e))?;
        self.formatter
            .write_string(&mut self.writer, key)
            .map_err(|e| Error::Io(e))?;
        self.formatter
            .end_key(&mut self.writer)
            .map_err(|e| Error::Io(e))?;

        self.formatter
            .begin_value(&mut self.writer)
            .map_err(|e| Error::Io(e))?;
        value.serialize(&mut **self)?;
        self.formatter
            .end_value(&mut self.writer)
            .map_err(|e| Error::Io(e))?;

        self.key_stack.pop();
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        self.formatter
            .end_object(&mut self.writer)
            .map_err(|e| Error::Io(e))
    }
}

impl<'a, W: Write, F: Formatter> serde::ser::SerializeStructVariant for &'a mut Serializer<W, F> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<Self::Ok> {
        serde::ser::SerializeStruct::serialize_field(self, key, value)
    }

    fn end(self) -> Result<Self::Ok> {
        self.formatter
            .end_object(&mut self.writer)
            .map_err(|e| Error::Io(e))?;
        self.formatter
            .end_value(&mut self.writer)
            .map_err(|e| Error::Io(e))?;

        self.key_stack.pop();
        self.formatter
            .end_object(&mut self.writer)
            .map_err(|e| Error::Io(e))?;
        Ok(())
    }
}
