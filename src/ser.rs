#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use crate::error::{Error, Result};
use serde::ser::{self, Serialize};

mod types {
    pub const INTEGER: i32 = 11_i32;
    pub const BLOCK: i32 = 5_i32;
}

pub struct Serializer {
    output: Vec<u8>,
    length: i32,
}

impl Serializer {
    pub fn new() -> Self {
        Serializer {
            output: Vec::new(),
            length: 0,
        }
    }
}

pub fn to_bytes<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    let mut header = Vec::from([0x52, 0x45, 0x44, 0x42, 0x49, 0x4E, // "REDBIN"
            0x02, // version
            0x00, // flags
            0x01, 0x00, 0x00, 0x00]); // length (number of records))

    let mut serializer = Serializer::new();
    value.serialize(&mut serializer)?;
    header.append(&mut Vec::from((serializer.output.len() as i32).to_le_bytes())); // size of payload
    Ok([&header[..], &serializer.output[..]].concat())
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<()> {
        unimplemented!("TODO");
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_i32(v as i32)
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_i32(v as i32)
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.output.append(&mut Vec::from(types::INTEGER.to_le_bytes()));
        self.output.append(&mut Vec::from(v.to_le_bytes()));
        //println!("i32: {:?}", v.to_le_bytes()); // DEBUG
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        if v > (i32::MAX as i64)
                || v < (i32::MIN as i64) {
            Err(Error::Message(String::from("32-bit integer! limit exceeded")))
        } else {
            self.serialize_i32(v as i32)
        }
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.serialize_i32(v as i32)
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.serialize_i32(v as i32)
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        if v > (i32::MAX as u64) {
            Err(Error::Message(String::from("32-bit integer! limit exceeded")))
        } else {
            self.serialize_i32(v as i32)
        }
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        unimplemented!("TODO");
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        unimplemented!("TODO");
    }

    fn serialize_char(self, v: char) -> Result<()> {
        unimplemented!("TODO");
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        unimplemented!("TODO");
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        unimplemented!("TODO");
    }

    fn serialize_none(self) -> Result<()> {
        unimplemented!("TODO");
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!("TODO");
    }

    fn serialize_unit(self) -> Result<()> {
        unimplemented!("TODO");
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        unimplemented!("TODO");
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        unimplemented!("TODO");
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!("TODO");
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!("TODO");
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.output.append(&mut Vec::from(types::BLOCK.to_le_bytes()));
        self.output.append(&mut Vec::from([0x00, 0x00, 0x00, 0x00])); // position block on start
        self.output.append(&mut Vec::from((len as i32).to_le_bytes()));
        self.length = len as i32;
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        unimplemented!("TODO");
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        unimplemented!("TODO");
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        unimplemented!("TODO");
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct> {
        unimplemented!("TODO");
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        unimplemented!("TODO");
    }
}

impl<'a> ser::SerializeSeq for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let mut serializer = Serializer::new();
        value.serialize(&mut serializer)?;
        self.output.append(&mut serializer.output);
        self.length += 1;
        Ok(())
    }

    fn end(self) -> Result<()> {
        let mut header = Vec::new();
        
        header.append(&mut Vec::from(types::BLOCK.to_le_bytes()));
        header.append(&mut Vec::from([0x00, 0x00, 0x00, 0x00])); // position block on start
        header.append(&mut Vec::from(self.length.to_le_bytes()));

        self.output.splice(0..0, header);
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let mut serializer = Serializer::new();
        value.serialize(&mut serializer)?;
        self.output.append(&mut serializer.output);
        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!("TODO");
    }

    fn end(self) -> Result<()> {
        unimplemented!("TODO");
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!("TODO");
    }

    fn end(self) -> Result<()> {
        unimplemented!("TODO");
    }
}

impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!("TODO");
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!("TODO");
    }

    fn end(self) -> Result<()> {
        unimplemented!("TODO");
    }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!("TODO");
    }

    fn end(self) -> Result<()> {
        unimplemented!("TODO");
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!("TODO");
    }

    fn end(self) -> Result<()> {
        unimplemented!("TODO");
    }
}

////////////////////////////////////////////////////////////////////////////////

/*
Redbin values can be generated in Red:

>> rust-redbin-helper: function [value] [buf: copy #{}  save/as buf value 'redbin  foreach b buf [prin rejoin [", 0x"   copy/part  at mold to binary! b 9  2]]  print ""]
== func [value /local buf b][buf: copy #{} save/as buf value 'redbin foreach b buf [prin re...
>> rust-redbin-helper 55
, 0x52, 0x45, 0x44, 0x42, 0x49, 0x4E, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x37, 0x00, 0x00, 0x00
*/

#[cfg(test)]
mod tests {
    use super::to_bytes;
    use serde_derive::Serialize;
    
    #[test]
    fn test_seq() {
        let i: (i8, i16, u32, &[u8]) = (-2, 299, 66666, &[5, 6]);
        let expected = &[0x52, 0x45, 0x44, 0x42, 0x49, 0x4E, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x0B, 0x00, 0x00, 0x00, 0xFE, 0xFF, 0xFF, 0xFF, 0x0B, 0x00, 0x00, 0x00, 0x2B, 0x01, 0x00, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x6A, 0x04, 0x01, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00];
        assert_eq!(to_bytes(&i).unwrap(), expected);
    }

}
