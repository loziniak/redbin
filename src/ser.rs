#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use crate::error::{Error, Result};
use serde::ser::{self, Serialize};
use crate::iconv_tools::Ic;

mod types {
    pub const LOGIC: i32 = 0x04_i32;
    pub const BLOCK: i32 = 0x05_i32;
    pub const STRING: i32 = 0x07_i32;
    pub const CHAR: i32 = 0x0A_i32;
    pub const INTEGER: i32 = 0x0B_i32;
    pub const FLOAT: i32 = 0x0C_i32;
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

impl<'a> ser::SerializeSeq for &'a mut Serializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let mut serializer = Serializer::new_with(self.ic);
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

impl<'a> ser::SerializeTuple for &'a mut Serializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let mut serializer = Serializer::new_with(self.ic);
        value.serialize(&mut serializer)?;
        self.output.append(&mut serializer.output);
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
        unimplemented!("TODO");
    }

    fn end(self) -> Result<()> {
        unimplemented!("TODO");
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer<'_> {
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

impl<'a> ser::SerializeMap for &'a mut Serializer<'_> {
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

impl<'a> ser::SerializeStruct for &'a mut Serializer<'_> {
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

impl<'a> ser::SerializeStructVariant for &'a mut Serializer<'_> {
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
        let i: (i8, i16, u32, &[u8], bool, f64, f32, &str, &str, &str, char, char) = (-2, 299, 66666, &[5, 6], true, 122234.23425, 12.5, "aa", "ą", "💖", 'a', '💖');
        
        // rust-redbin-helper reduce [-2 299 66666 [5 6] yes 122234.23425 12.5 "aa" "ą" "💖" #"a" #"ą"]
        let expected = &[0x52, 0x45, 0x44, 0x42, 0x49, 0x4E, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0xA0, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x00, 0x00, 0x00, 0x0B, 0x00, 0x00, 0x00, 0xFE, 0xFF, 0xFF, 0xFF, 0x0B, 0x00, 0x00, 0x00, 0x2B, 0x01, 0x00, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x6A, 0x04, 0x01, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x0C, 0x00, 0x00, 0x00, 0xA3, 0xD7, 0xFD, 0x40, 0x91, 0xED, 0x7C, 0xBF, 0x0C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x29, 0x40, 0x00, 0x00, 0x00, 0x00, 0x07, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x61, 0x61, 0x00, 0x00, 0x07, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x05, 0x01, 0x00, 0x00, 0x07, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x96, 0xF4, 0x01, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x61, 0x00, 0x00, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x96, 0xF4, 0x01, 0x00];
        
        assert_eq!(to_bytes(&i).unwrap(), expected);
    }

}
