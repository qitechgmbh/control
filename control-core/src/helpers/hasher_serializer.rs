use serde::Serialize;
use serde::ser::{
    SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
    SerializeTupleStruct, SerializeTupleVariant, Serializer,
};
use std::error::Error;
use std::fmt::Display;
use std::hash::{DefaultHasher, Hasher};

#[derive(Debug)]
pub struct HashSerializerError {}

impl std::fmt::Display for HashSerializerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self))
        // f.debug_struct("HashSerialiyerError")
        // .finish()
    }
}

impl Error for HashSerializerError {}

impl serde::ser::Error for HashSerializerError {
    fn custom<T>(_msg: T) -> Self
    where
        T: Display,
    {
        Self {}
    }
}

pub struct HashSerializer<'a, H: Hasher>(&'a mut H);
impl<H: Hasher> SerializeSeq for HashSerializer<'_, H> {
    type Ok = ();
    type Error = HashSerializerError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(HashSerializer(self.0))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<H: Hasher> SerializeTuple for HashSerializer<'_, H> {
    type Ok = ();
    type Error = HashSerializerError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(HashSerializer(self.0))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<H: Hasher> SerializeTupleStruct for HashSerializer<'_, H> {
    type Ok = ();
    type Error = HashSerializerError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(HashSerializer(self.0))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<H: Hasher> SerializeTupleVariant for HashSerializer<'_, H> {
    type Ok = ();
    type Error = HashSerializerError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(HashSerializer(self.0))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<H: Hasher> SerializeMap for HashSerializer<'_, H> {
    type Ok = ();
    type Error = HashSerializerError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(HashSerializer(self.0))
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(HashSerializer(self.0))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<H: Hasher> SerializeStruct for HashSerializer<'_, H> {
    type Ok = ();
    type Error = HashSerializerError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        // Optional: hash the field name to avoid collisions
        self.0.write(key.as_bytes());
        value.serialize(HashSerializer(self.0))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<H: Hasher> SerializeStructVariant for HashSerializer<'_, H> {
    type Ok = ();
    type Error = HashSerializerError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.0.write(key.as_bytes());
        value.serialize(HashSerializer(self.0))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<H: Hasher> Serializer for HashSerializer<'_, H> {
    type Ok = ();
    type Error = HashSerializerError;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<(), Self::Error> {
        self.0.write_u8(v as u8);
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<(), Self::Error> {
        self.0.write_i8(v);
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<(), Self::Error> {
        self.0.write_i16(v);
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<(), Self::Error> {
        self.0.write_i32(v);
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<(), Self::Error> {
        self.0.write_i64(v);
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<(), Self::Error> {
        self.0.write_u8(v);
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<(), Self::Error> {
        self.0.write_u16(v);
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<(), Self::Error> {
        self.0.write_u32(v);
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<(), Self::Error> {
        self.0.write_u64(v);
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<(), Self::Error> {
        self.0.write_u32(v.to_bits());
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<(), Self::Error> {
        self.0.write_u64(v.to_bits());
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<(), Self::Error> {
        let mut buf = [0; 4];
        self.0.write(v.encode_utf8(&mut buf).as_bytes());
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<(), Self::Error> {
        self.0.write(v.as_bytes());
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<(), Self::Error> {
        self.0.write(v);
        Ok(())
    }

    fn serialize_none(self) -> Result<(), Self::Error> {
        self.0.write_u8(0); // type tag for None
        Ok(())
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<(), Self::Error> {
        self.0.write_u8(1); // type tag for Some
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<(), Self::Error> {
        self.0.write_u8(0);
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<(), Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<(), Self::Error> {
        self.0.write_u32(variant_index);
        Ok(())
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        self.0.write_u32(variant_index);
        value.serialize(self)
    }

    // For sequences, tuples, maps, structs, just return self for now
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(self)
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(self)
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(self)
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.0.write_u32(variant_index);
        Ok(self)
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(self)
    }
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.0.write_u32(variant_index);
        Ok(self)
    }
}

// A helper that uses Serialize to feed into a Hasher without allocating a buffer:
pub fn hash_with_serde_model<T: Serialize>(value: T) -> u64 {
    let mut h = DefaultHasher::new();
    let ser = HashSerializer(&mut h);
    value.serialize(ser).expect("serialize to hasher failed");
    h.finish()
}
