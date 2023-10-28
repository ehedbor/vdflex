use super::formatter::{FormatOpts, Formatter, PrettyFormatter};
use crate::{Error, Result};
use serde::Serialize;
use std::io::Write;

pub struct Serializer<W, F = PrettyFormatter>
where
    W: Write,
    F: Formatter,
{
    writer: W,
    formatter: F,
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
        Self { writer, formatter }
    }
}

impl<W, F> serde::Serializer for Serializer<W, F>
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

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_i128(self, _v: i128) -> Result<Self::Ok> {
        Err(Error::UnsupportedType("i128".to_string()))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_u128(self, _v: u128) -> Result<Self::Ok> {
        Err(Error::UnsupportedType("u128".to_string()))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(mut self, v: &str) -> Result<Self::Ok> {
        self.formatter
            .write_value(&mut self.writer, v)
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
        Err(Error::UnsupportedType("unit".to_string()))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        Err(Error::UnsupportedType("unit struct".to_string()))
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
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        Err(Error::UnsupportedType("enum newtype variant".to_string()))
        // self.formatter
        //     .begin_object(&mut self.writer)
        //     .map_err(|e| Error::Io(e))?;
        // {
        //     self.formatter
        //         .begin_key(&mut self.writer)
        //         .map_err(|e| Error::Io(e))?;
        //     self.serialize_str(variant)?;
        //     self.formatter
        //         .end_key(&mut self.writer)
        //         .map_err(|e| Error::Io(e))?;
        //
        //     value.serialize(self)?;
        // }
        // self.formatter
        //     .end_object(&mut self.writer)
        //     .map_err(|e| Error::Io(e))?;
        //
        // Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(Error::UnsupportedType("sequence".to_string()))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(Error::UnsupportedType("tuple".to_string()))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(Error::UnsupportedType("tuple struct".to_string()))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::UnsupportedType("enum tuple variant".to_string()))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::UnsupportedType("map".to_string()))
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(Error::UnsupportedType("struct".to_string()))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::UnsupportedType("enum struct variant".to_string()))
    }
}

impl<W, F> serde::ser::SerializeSeq for Serializer<W, F>
where
    W: Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::UnsupportedType("sequence".to_string()))
    }

    fn end(self) -> Result<Self::Ok> {
        Err(Error::UnsupportedType("sequence".to_string()))
    }
}

impl<W, F> serde::ser::SerializeTuple for Serializer<W, F>
where
    W: Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::UnsupportedType("tuple".to_string()))
    }

    fn end(self) -> Result<Self::Ok> {
        Err(Error::UnsupportedType("tuple".to_string()))
    }
}

impl<W, F> serde::ser::SerializeTupleStruct for Serializer<W, F>
where
    W: Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::UnsupportedType("tuple struct".to_string()))
    }

    fn end(self) -> Result<Self::Ok> {
        Err(Error::UnsupportedType("tuple struct".to_string()))
    }
}

impl<W, F> serde::ser::SerializeTupleVariant for Serializer<W, F>
where
    W: Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::UnsupportedType("enum tuple variant".to_string()))
    }

    fn end(self) -> Result<Self::Ok> {
        Err(Error::UnsupportedType("enum tuple variant".to_string()))
    }
}

impl<W, F> serde::ser::SerializeMap for Serializer<W, F>
where
    W: Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, _key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::UnsupportedType("map".to_string()))
    }

    fn serialize_value<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::UnsupportedType("map".to_string()))
    }

    fn end(self) -> Result<Self::Ok> {
        Err(Error::UnsupportedType("map".to_string()))
    }
}

impl<W, F> serde::ser::SerializeStruct for Serializer<W, F>
where
    W: Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::UnsupportedType("struct".to_string()))
    }

    fn end(self) -> Result<Self::Ok> {
        Err(Error::UnsupportedType("struct".to_string()))
    }
}

impl<W, F> serde::ser::SerializeStructVariant for Serializer<W, F>
where
    W: Write,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::UnsupportedType("enum struct variant".to_string()))
    }

    fn end(self) -> Result<Self::Ok> {
        Err(Error::UnsupportedType("enum struct variant".to_string()))
    }
}
