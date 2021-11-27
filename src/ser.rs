#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use crate::error::{Error, Result};
use serde::ser::{self, Serialize};
use crate::iconv_tools::Ic;

mod types {
    pub const NONE: i32 = 0x03_i32;
    pub const LOGIC: i32 = 0x04_i32;
    pub const BLOCK: i32 = 0x05_i32;
    pub const PAREN: i32 = 0x06_i32;
    pub const STRING: i32 = 0x07_i32;
    pub const CHAR: i32 = 0x0A_i32;
    pub const INTEGER: i32 = 0x0B_i32;
    pub const FLOAT: i32 = 0x0C_i32;
    pub const BYTES: i32 = 0x29_i32;
}

pub struct Serializer<'b> {
    output: Vec<u8>,
    length: i32,
    ic: &'b mut Ic,
}

impl<'b> Serializer<'b> {
    pub fn new_with(ic1: &'b mut Ic) -> Self {
        Serializer {
            output: Vec::new(),
            length: 0,
            ic: ic1
        }
    }

    fn append_any_block_header(vec: &mut Vec<u8>, length: i32, paren: bool) -> () {
        let t = if paren {types::PAREN} else {types::BLOCK};
        vec.append(&mut Vec::from(t.to_le_bytes()));
        vec.append(&mut Vec::from([0x00, 0x00, 0x00, 0x00])); // position block on start
        vec.append(&mut Vec::from(length.to_le_bytes()));
    }

    fn block_header_with(&mut self, len: usize) -> () {
        Self::append_any_block_header(&mut self.output, len as i32, false);
        self.length = len as i32;
    }

    fn variant_header(&mut self, with_value: bool) -> () {
        let length = if with_value {2} else {1};
        Self::append_any_block_header(&mut self.output, length, true);
        self.length = length;
    }

    fn elements<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let mut serializer = Serializer::new_with(self.ic);
        value.serialize(&mut serializer)?;
        self.output.append(&mut serializer.output);
        Ok(())
    }

    fn prepend_block_header(&mut self) -> () {
        let mut header = Vec::new();
        Self::append_any_block_header(&mut header, self.length, false);
        self.output.splice(0..0, header);
    }
}

pub fn to_bytes<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    to_bytes_with(&mut Ic::new(), value)
}

pub fn to_bytes_with<T>(ic: &mut Ic, value: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    let mut header = Vec::from([0x52, 0x45, 0x44, 0x42, 0x49, 0x4E, // "REDBIN"
            0x02, // version
            0x00, // flags
            0x01, 0x00, 0x00, 0x00]); // length (number of records))

    let mut serializer = Serializer::new_with(ic);
    value.serialize(&mut serializer)?;
    header.append(&mut Vec::from((serializer.output.len() as i32).to_le_bytes())); // size of payload
    Ok([&header[..], &serializer.output[..]].concat())
}

impl<'a> ser::Serializer for &'a mut Serializer<'_> {
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
        self.output.append(&mut Vec::from(types::LOGIC.to_le_bytes()));
        self.output.append(&mut Vec::from((v as i32).to_le_bytes()));
        Ok(())
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
        self.serialize_f64(v as f64)
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.output.append(&mut Vec::from(types::FLOAT.to_le_bytes()));
        let mut bytes = Vec::from(v.to_le_bytes());

        // swap words
        bytes.append(&mut Vec::from(&bytes[0..4]));
        bytes = Vec::from(&bytes[4..12]);

        // Optional padding at the beginning is not added.
        // Red's "load/as [...] 'redbin" command accepts data without padding.
        self.output.append(&mut bytes);
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.output.append(&mut Vec::from(types::CHAR.to_le_bytes()));
        let bytes = self.ic.ucs4_encode(v.encode_utf8(&mut [0x00; 4]))
            .map_err(|e| Error::Message(e.to_string()))?;
        self.output.append(&mut Vec::from(bytes));
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        let mut header = Vec::from(types::STRING.to_le_bytes());
        let len = v.chars().count() as i32;
        let (mut encoded, padding) = if len == (v.len() as i32) { // ASCII
            header[1] = 0x01; // 1-byte characters, ASCII
            (Vec::from(v),
                ((4 - (len % 4)) % 4) as usize)
        } else { // Unicode
            header[1] = 0x04; // 4-byte characters, UCS-4
            (self.ic.ucs4_encode(v)
                .map_err(|e| Error::Message(e.to_string()))?,
                0 as usize)
        };
        header.append(&mut Vec::from([0x00; 4])); // head position
        header.append(&mut Vec::from(len.to_le_bytes()));

        self.output.append(&mut header);
        
        self.output.append(&mut encoded);
        
        let mut p = Vec::from([0x00; 4]);
        p.resize(padding, 0x00);
        self.output.append(&mut p);
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        let mut header = Vec::from(types::BYTES.to_le_bytes());
        let len = v.len() as i32;
        header[1] = 0x01; // unit
        let mut encoded = Vec::from(v);
        let padding = ( (4 - (len % 4)) % 4 ) as usize;
        header.append(&mut Vec::from([0x00; 4])); // head position
        header.append(&mut Vec::from(len.to_le_bytes()));

        self.output.append(&mut header);
        
        self.output.append(&mut encoded);
        
        let mut p = Vec::from([0x00; 4]);
        p.resize(padding, 0x00);
        self.output.append(&mut p);
        Ok(())
    }

    fn serialize_none(self) -> Result<()> {
        self.output.append(&mut Vec::from(types::NONE.to_le_bytes()));
        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        self.serialize_none()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_none()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.variant_header(false);
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
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
        self.variant_header(true);
        self.serialize_str(variant)?;
        value.serialize(self)?;
        Ok(())
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.block_header_with(len);
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.block_header_with(len);
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.variant_header(true);
        self.serialize_str(variant)?;
        self.block_header_with(len);
        Ok(self)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(self)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct> {
        self.block_header_with(len * 2);
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.variant_header(true);
        self.serialize_str(variant)?;
        self.block_header_with(len * 2);
        Ok(self)
    }
}

impl<'a> ser::SerializeSeq for &'a mut Serializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.elements(value)?;
        self.length += 1;
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.prepend_block_header();
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for &'a mut Serializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.elements(value)?;
        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut Serializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.elements(value)?;
        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.elements(value)?;
        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeMap for &'a mut Serializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.elements(key)?;
        self.length += 1;
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.elements(value)?;
        self.length += 1;
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.prepend_block_header();
        Ok(())
    }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.elements(key)?;
        self.elements(value)?;
        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.elements(key)?;
        self.elements(value)?;
        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
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
    use serde_bytes::ByteBuf;
    
    #[test]
    fn test_seq() {
        let i: (i8, i16, u32, &[u8], bool, f64, f32, &str, &str, &str, char, char, ByteBuf) = (-2, 299, 66666, &[5, 6], true, 122234.23425, 12.5, "aa", "ą", "💖", 'a', '💖', ByteBuf::from([0xCA, 0xFE]));
        
        // rust-redbin-helper reduce [-2 299 66666 [5 6] yes 122234.23425 12.5 "aa" "ą" "💖" #"a" #"💖" #{CAFE}]
        let expected = &[0x52, 0x45, 0x44, 0x42, 0x49, 0x4E, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0xB0, 0x00, 0x00, 0x00, 
        0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0D, 0x00, 0x00, 0x00, 
            0x0B, 0x00, 0x00, 0x00, 0xFE, 0xFF, 0xFF, 0xFF,
            0x0B, 0x00, 0x00, 0x00, 0x2B, 0x01, 0x00, 0x00,
            0x0B, 0x00, 0x00, 0x00, 0x6A, 0x04, 0x01, 0x00,
            0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00,
                0x0B, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00,
                0x0B, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00,
            0x04, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
            0x0C, 0x00, 0x00, 0x00, 0xA3, 0xD7, 0xFD, 0x40, 0x91, 0xED, 0x7C, 0xBF,
            0x0C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x29, 0x40, 0x00, 0x00, 0x00, 0x00,
            0x07, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x61, 0x61, 0x00, 0x00,
            0x07, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x05, 0x01, 0x00, 0x00,
            0x07, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x96, 0xF4, 0x01, 0x00,
            0x0A, 0x00, 0x00, 0x00, 0x61, 0x00, 0x00, 0x00,
            0x0A, 0x00, 0x00, 0x00, 0x96, 0xF4, 0x01, 0x00,
            0x29, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0xCA, 0xFE, 0x00, 0x00];
        
        assert_eq!(to_bytes(&i).unwrap(), expected);

        // rust-redbin-helper none
        assert_eq!(to_bytes::<()>(&()).unwrap(),
            &[0x52, 0x45, 0x44, 0x42, 0x49, 0x4E, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 
            0x03, 0x00, 0x00, 0x00]);

        // rust-redbin-helper none
        assert_eq!(to_bytes::<Option<i32>>(&None).unwrap(),
            &[0x52, 0x45, 0x44, 0x42, 0x49, 0x4E, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00,
            0x03, 0x00, 0x00, 0x00]);

        // rust-redbin-helper 123
        assert_eq!(to_bytes::<Option<i32>>(&Some(123)).unwrap(),
            &[0x52, 0x45, 0x44, 0x42, 0x49, 0x4E, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00,
            0x0B, 0x00, 0x00, 0x00, 0x7B, 0x00, 0x00, 0x00]);


        #[derive(Serialize, std::cmp::PartialEq, Debug)]
        struct NothingSpecial;
        let test_something = NothingSpecial {};

        // rust-redbin-helper none
        assert_eq!(to_bytes(&test_something).unwrap(),
            &[0x52, 0x45, 0x44, 0x42, 0x49, 0x4E, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00,
            0x03, 0x00, 0x00, 0x00]);


        #[derive(Serialize, std::cmp::PartialEq, Debug)]
        struct Num(i32);
        let fifteen = Num(15);

        // rust-redbin-helper 15
        assert_eq!(to_bytes(&fifteen).unwrap(),
            &[0x52, 0x45, 0x44, 0x42, 0x49, 0x4E, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00,
            0x0B, 0x00, 0x00, 0x00, 0x0F, 0x00, 0x00, 0x00]);


        #[derive(Serialize, std::cmp::PartialEq, Debug)]
        struct Color(u8, u8, u8);
        let red = Color(255, 0, 0);

        // rust-redbin-helper [255 0 0]
        assert_eq!(to_bytes(&red).unwrap(),
            &[0x52, 0x45, 0x44, 0x42, 0x49, 0x4E, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x24, 0x00, 0x00, 0x00,
            0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00,
                0x0B, 0x00, 0x00, 0x00, 0xFF, 0x00, 0x00, 0x00,
                0x0B, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x0B, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);


        let mut test_map: std::collections::BTreeMap<String, f64> = std::collections::BTreeMap::new();
        test_map.insert(String::from("a"), 12.5);
        test_map.insert(String::from("b"), 100.1);

        // rust-redbin-helper ["a" 12.5 "b" 100.1]
        assert_eq!(to_bytes(&test_map).unwrap(),
            &[0x52, 0x45, 0x44, 0x42, 0x49, 0x4E, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x44, 0x00, 0x00, 0x00,
            0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00,
                0x07, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x61, 0x00, 0x00, 0x00, 
                0x0C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x29, 0x40, 0x00, 0x00, 0x00, 0x00,
                0x07, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x62, 0x00, 0x00, 0x00, 
                0x0C, 0x00, 0x00, 0x00, 0x66, 0x06, 0x59, 0x40, 0x66, 0x66, 0x66, 0x66]);


        #[derive(Serialize, std::cmp::PartialEq, Debug)]
        struct WhatNot { a: f64, b: String }
        let wtf = WhatNot { a: 12.5, b: String::from("sdf") };

        // rust-redbin-helper ["a" 12.5 "b" "sdf"]
        assert_eq!(to_bytes(&wtf).unwrap(),
            &[0x52, 0x45, 0x44, 0x42, 0x49, 0x4E, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x48, 0x00, 0x00, 0x00,
            0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00,
                0x07, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x61, 0x00, 0x00, 0x00, 
                0x0C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x29, 0x40, 0x00, 0x00, 0x00, 0x00,
                0x07, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x62, 0x00, 0x00, 0x00, 
                0x07, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x73, 0x64, 0x66, 0x00]);


        #[derive(Serialize, std::cmp::PartialEq, Debug)]
        enum E {
            Unit,
            Newtype(u32),
            Tuple(u32, u32),
            Struct { a: u32 },
        }
        let enum_test_tuple = (E::Unit, E::Newtype(1), E::Tuple(1, 2), E::Struct { a: 1 });

        // rust-redbin-helper [ ("Unit") ("Newtype" 1) ("Tuple" [1 2]) ("Struct" ["a" 1]) ]
        assert_eq!(to_bytes(&enum_test_tuple).unwrap(),
            &[0x52, 0x45, 0x44, 0x42, 0x49, 0x4E, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0xD0, 0x00, 0x00, 0x00,
            0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00,
                0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
                    0x07, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x55, 0x6E, 0x69, 0x74,
                0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00,
                    0x07, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00, 0x4E, 0x65, 0x77, 0x74, 0x79, 0x70, 0x65, 0x00,
                    0x0B, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
                0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00,
                    0x07, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x54, 0x75, 0x70, 0x6C, 0x65, 0x00, 0x00, 0x00,
                    0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00,
                        0x0B, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
                        0x0B, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00,
                0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00,
                    0x07, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x53, 0x74, 0x72, 0x75, 0x63, 0x74, 0x00, 0x00,
                    0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00,
                        0x07, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x61, 0x00, 0x00, 0x00,
                        0x0B, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00]);

    }

}
