use super::formatter::{FormatOpts, Formatter, PrettyFormatter};
use crate::{Error, Result};
use serde::ser::Impossible;
use serde::Serialize;
use std::borrow::Cow;
use std::io::Write;

pub struct Serializer<W, F = PrettyFormatter>
where
    W: Write,
    F: Formatter,
{
    writer: W,
    formatter: F,
    key_stack: Vec<Cow<'static, str>>,
}

impl<W> Serializer<W, PrettyFormatter>
where
    W: Write,
{
    /// Creates a new KeyValues serializer using an appropriate formatter.
    pub fn new(writer: W) -> Self {
        Self::custom(writer, PrettyFormatter::new())
    }

    /// Creates a new KeyValues serializer using a `PrettyFormatter` with the given options.
    pub fn pretty(writer: W, opts: FormatOpts) -> Self {
        Self::custom(writer, PrettyFormatter::with_opts(opts))
    }
}

impl<W, F> Serializer<W, F>
where
    W: Write,
    F: Formatter,
{
    pub fn custom(writer: W, formatter: F) -> Self {
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

impl<'a, W, F> serde::Serializer for &'a mut Serializer<W, F>
where
    W: Write,
    F: Formatter,
{
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
        self.serialize_str("")
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
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

    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: Serialize,
    {
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
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.key_stack.push(Cow::Borrowed(variant));
        self.formatter
            .begin_object(&mut self.writer)
            .map_err(|e| Error::Io(e))?;
        self.serialize_seq(Some(len))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        self.formatter
            .begin_object(&mut self.writer)
            .map_err(|e| Error::Io(e))?;
        Ok(self)
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
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

struct MapKeySerializer<'a, W, F>
where
    W: Write,
    F: Formatter,
{
    serializer: &'a mut Serializer<W, F>,
}

impl<'a, W, F> serde::Serializer for MapKeySerializer<'a, W, F>
where
    W: Write,
    F: Formatter,
{
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

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
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

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
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

impl<'a, W, F> serde::ser::SerializeSeq for &'a mut Serializer<W, F>
where
    W: Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        let key = self.key_stack.last().ok_or(Error::RootLevelSequence)?;

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

impl<'a, W, F> serde::ser::SerializeTuple for &'a mut Serializer<W, F>
where
    W: Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl<'a, W, F> serde::ser::SerializeTupleStruct for &'a mut Serializer<W, F>
where
    W: Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        self.formatter
            .end_object(&mut self.writer)
            .map_err(|e| Error::Io(e))?;
        self.key_stack.pop();
        serde::ser::SerializeSeq::end(self)
    }
}

impl<'a, W, F> serde::ser::SerializeTupleVariant for &'a mut Serializer<W, F>
where
    W: Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        self.key_stack.pop();
        serde::ser::SerializeSeq::end(self)
    }
}

impl<'a, W, F> serde::ser::SerializeMap for &'a mut Serializer<W, F>
where
    W: Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        let ser = MapKeySerializer { serializer: self };
        key.serialize(ser)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
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

impl<'a, W, F> serde::ser::SerializeStruct for &'a mut Serializer<W, F>
where
    W: Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
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

impl<'a, W, F> serde::ser::SerializeStructVariant for &'a mut Serializer<W, F>
where
    W: Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        serde::ser::SerializeStruct::serialize_field(self, key, value)
    }

    fn end(self) -> Result<Self::Ok> {
        self.formatter
            .end_value(&mut self.writer)
            .map_err(|e| Error::Io(e))?;
        self.formatter
            .end_object(&mut self.writer)
            .map_err(|e| Error::Io(e))?;

        self.key_stack.pop();
        self.formatter
            .end_object(&mut self.writer)
            .map_err(|e| Error::Io(e))?;
        Ok(())
    }
}
