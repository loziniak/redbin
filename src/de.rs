#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use crate::error::{Error, Result};
use serde::de::{
    self, Deserialize, DeserializeSeed, EnumAccess, IntoDeserializer,
    MapAccess, SeqAccess, VariantAccess, Visitor,
};
use std::ops::{AddAssign, MulAssign, Neg};
use std::convert::TryInto;
use crate::iconv_tools::Ic;

mod types {
    pub const NONE: u8 = 0x03;
    pub const LOGIC: u8 = 0x04;
    pub const BLOCK: u8 = 0x05;
    pub const STRING: u8 = 0x07;
    pub const CHAR: u8 = 0x0A;
    pub const INTEGER: u8 = 0x0B;
    pub const FLOAT: u8 = 0x0C;
    pub const BINARY: u8 = 0x29;
}

pub struct Deserializer<'de, 'b> {
    input: &'de [u8],
    ic: &'b mut Ic,
}

impl<'de, 'b> Deserializer<'de, 'b> {
    #[allow(clippy::should_implement_trait)]
    pub fn from_bytes(input: &'de [u8], ic: &'b mut Ic) -> Self {
        Deserializer { input, ic }
    }
}

pub fn from_bytes<'a, T>(s: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut ic = Ic::new();
    let mut deserializer = Deserializer::from_bytes(s, &mut ic);
    deserializer.parse_header()?;
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.input.is_empty() {
        Ok(t)
    } else {
        Err(Error::TrailingBytes)
    }
}


impl<'de, 'b> Deserializer<'de, 'b> {

    fn parse_padding(&mut self) -> Result<()> {
        while self.input.len() > 0 && self.input[0] == 0x00 {
            self.input = &self.input[1..];
        }
        Ok(())
    }
    
    fn parse_header(&mut self) -> Result<()> {
        let header_len = [0x52, 0x45, 0x44, 0x42, 0x49, 0x4E, // "REDBIN"
            0x02, // version
            0x00, // flags
            0x01, 0x00, 0x00, 0x00,  // length (number of records)
            0x08, 0x00, 0x00, 0x00]  // size of payload
            .len();
        self.input = &self.input[header_len..];
        Ok(())
    }
    
    fn parse_integer(&mut self) -> Result<i32> {
        self.parse_padding()?;
        if &self.input[..4] == &[types::INTEGER, 0x00, 0x00, 0x00] {
            let bytes = &self.input[4..8];
            //println!("bytes: {:?}", bytes); // DEBUG
            self.input = &self.input[8..];
            Ok(i32::from_le_bytes(bytes.try_into().unwrap()))
        } else {
            Err(Error::ExpectedInteger)
        }
    }
    
    fn parse_block_header(&mut self) -> Result<i32> {
        self.parse_padding()?;
        if &self.input[..4] == &[types::BLOCK, 0x00, 0x00, 0x00] {
            let len = &self.input[8..12];
            self.input = &self.input[12..];
            Ok(i32::from_le_bytes(len.try_into().unwrap()))
        } else {
            Err(Error::ExpectedBlock)
        }
    }

    fn parse_logic(&mut self) -> Result<bool> {
        self.parse_padding()?;
        if &self.input[..4] == &[types::LOGIC, 0x00, 0x00, 0x00] {
            let bytes = &self.input[4..8];
            self.input = &self.input[8..];
            Ok(i32::from_le_bytes(bytes.try_into().unwrap()) != 0)
        } else {
            Err(Error::ExpectedLogic)
        }
    }

    fn parse_float(&mut self) -> Result<f64> {
        self.parse_padding()?;
        if &self.input[..4] == &[types::FLOAT, 0x00, 0x00, 0x00] {
            let bytes = [&self.input[8..12], &self.input[4..8]].concat(); // swap words
            self.input = &self.input[12..];
            Ok(f64::from_le_bytes(bytes.try_into().unwrap()))
        } else {
            Err(Error::ExpectedFloat)
        }
    }

    fn parse_string(&mut self) -> Result<String> {
        self.parse_padding()?;
        if self.input[0] == types::STRING {
            let unit: usize = self.input[1] as usize;
            let refer: bool = (&self.input[2] & 0b_00001000) != 0;
            if refer {
                unimplemented!("Redbin references not supported yet.");
            } else {
                let head: usize = i32::from_le_bytes((&self.input[4..8]).try_into().unwrap()) as usize;
                let length: usize = i32::from_le_bytes((&self.input[8..12]).try_into().unwrap()) as usize;
                self.input = &self.input[12..];

                let n = length * unit;
                let bytes = &self.input[head..n];

                self.input = &self.input[n..];
                self.parse_padding()?;
                
                if unit == 1 {
                    String::from_utf8(bytes.to_vec())
                        .map_err(|e| Error::Message(e.to_string()))
                } else {
                    if unit == 2 {
                        self.ic.ucs2_decode(bytes)
                    } else {
                        self.ic.ucs4_decode(bytes)
                    }
                        .map_err(|e| Error::Message(e.to_string()))
                }
            }
        } else {
            Err(Error::ExpectedString)
        }
    }
    
    fn parse_char(&mut self) -> Result<char> {
        self.parse_padding()?;
        if self.input[0] == types::CHAR {
            let bytes = &self.input[4..8];
            self.input = &self.input[8..];
            self.ic.ucs4_decode(bytes)
                .map(|v| v.chars().next().unwrap())
                .map_err(|e| Error::Message(e.to_string()))
        } else {
            Err(Error::ExpectedChar)
        }
    }

    fn parse_binary(&mut self) -> Result<Vec<u8>> {
        self.parse_padding()?;
        if self.input[0] == types::BINARY {
            let unit: usize = self.input[1] as usize;
            let refer: bool = (&self.input[2] & 0b_00001000) != 0;
            if refer {
                unimplemented!("Redbin references not supported yet.");
            } else {
                let head: usize = i32::from_le_bytes((&self.input[4..8]).try_into().unwrap()) as usize;
                let length: usize = i32::from_le_bytes((&self.input[8..12]).try_into().unwrap()) as usize;
                self.input = &self.input[12..];

                let n = length * unit;
                let bytes = &self.input[head..n];

                self.input = &self.input[n..];
                self.parse_padding()?;
                
                if unit == 1 {
                    Ok(bytes.to_vec())
                } else {
                    unimplemented!("Unexpected unit size <> 1.");
                }
            }
        } else {
            Err(Error::ExpectedBinary)
        }
    }
    
    fn parse_none(&mut self) -> Result<()> {
        self.parse_padding()?;
        if self.input[0] == types::NONE {
            self.input = &self.input[4..];
            Ok(())
        } else {
            Err(Error::ExpectedNone)
        }
    }

}

impl<'de, 'a, 'b> de::Deserializer<'de> for &'a mut Deserializer<'de, 'b> {
    type Error = Error;

    // Look at the input data to decide what Serde data model type to
    // deserialize as. Not all data formats are able to support this operation.
    // Formats that support `deserialize_any` are known as self-describing.
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!();
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(self.parse_logic()?)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self.parse_integer()?;
        if v > (i8::MAX as i32)
                || v < (i8::MIN as i32) {
            Err(Error::Message(String::from("i8 limit exceeded")))
        } else {
            visitor.visit_i8(v as i8)
        }
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self.parse_integer()?;
        if v > (i16::MAX as i32)
                || v < (i16::MIN as i32) {
            Err(Error::Message(String::from("i16 limit exceeded")))
        } else {
            visitor.visit_i16(v as i16)
        }
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.parse_integer()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.parse_integer()? as i64)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self.parse_integer()?;
        if v > (u8::MAX as i32)
                || v < (u8::MIN as i32) {
            Err(Error::Message(String::from("u8 limit exceeded")))
        } else {
            visitor.visit_u8(v as u8)
        }
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self.parse_integer()?;
        if v > (u16::MAX as i32)
                || v < (u16::MIN as i32) {
            Err(Error::Message(String::from("u16 limit exceeded")))
        } else {
            visitor.visit_u16(v as u16)
        }
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self.parse_integer()?;
        if v < (u32::MIN as i32) {
            Err(Error::Message(String::from("u32 limit exceeded")))
        } else {
            visitor.visit_u32(v as u32)
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = self.parse_integer()?;
        if v < (u64::MIN as i32) {
            Err(Error::Message(String::from("u64 limit exceeded")))
        } else {
            visitor.visit_u64(v as u64)
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f32(self.parse_float()? as f32)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(self.parse_float()?)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_char(self.parse_char()?)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("Can only deserialize String.");
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.parse_string()?)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("Can only deserialize Vec<u8>.");
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_byte_buf(self.parse_binary()?)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.parse_padding()?;
        if self.input[0] == types::NONE {
            self.input = &self.input[4..];
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.parse_none()?;
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = self.parse_block_header()?;
        let value = visitor.visit_seq(BlockData::new(self, len))?;
        Ok(value)
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(_len, visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = self.parse_block_header()?;
        if len % 2 != 0 {
            return Err(Error::ExpectedEvenLength)
        }
        let value = visitor.visit_map(BlockData::new(self, len))?;
        Ok(value)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("TODO");
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!("TODO");
    }
}

// In order to handle commas correctly when deserializing a JSON array or map,
// we need to track whether we are on the first element or past the first
// element.
struct BlockData<'a, 'de: 'a, 'b> {
    de: &'a mut Deserializer<'de, 'b>,
    elements: i32,
}

impl<'a, 'de, 'b> BlockData<'a, 'de, 'b> {
    fn new(de: &'a mut Deserializer<'de, 'b>, len: i32) -> Self {
        BlockData { de, elements: len }
    }
}

// `SeqAccess` is provided to the `Visitor` to give it the ability to iterate
// through elements of the sequence.
impl<'de, 'a, 'b> SeqAccess<'de> for BlockData<'a, 'de, 'b> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if self.elements <= 0 {
            return Ok(None);
        }
        // Deserialize an array element.
        let v = seed.deserialize(&mut *self.de).map(Some)?;
        self.elements -= 1;
        Ok(v)
    }
    
    fn size_hint(&self) -> Option<usize> {
        Some(self.elements as usize)
    }
}

// `MapAccess` is provided to the `Visitor` to give it the ability to iterate
// through entries of the map.
impl<'de, 'a, 'b> MapAccess<'de> for BlockData<'a, 'de, 'b> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        if self.elements < 2 {
            return Ok(None);
        }
        let k = seed.deserialize(&mut *self.de).map(Some)?;
        self.elements -= 1;
        Ok(k)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        if self.elements < 1 {
            return Err(Error::NoMapValue);
        }
        let v = seed.deserialize(&mut *self.de)?;
        self.elements -= 1;
        Ok(v)
    }
}

struct Enum<'a, 'de: 'a, 'b> {
    de: &'a mut Deserializer<'de, 'b>,
}

impl<'a, 'de, 'b> Enum<'a, 'de, 'b> {
    fn new(de: &'a mut Deserializer<'de, 'b>) -> Self {
        Enum { de }
    }
}

// `EnumAccess` is provided to the `Visitor` to give it the ability to determine
// which variant of the enum is supposed to be deserialized.
//
// Note that all enum deserialization methods in Serde refer exclusively to the
// "externally tagged" enum representation.
impl<'de, 'a, 'b> EnumAccess<'de> for Enum<'a, 'de, 'b> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        // The `deserialize_enum` method parsed a `{` character so we are
        // currently inside of a map. The seed will be deserializing itself from
        // the key of the map.
        //let val = seed.deserialize(&mut *self.de)?;
        // Parse the colon separating map key from value.
        //if self.de.next_char()? == ':' {
            //Ok((val, self))
        //} else {
            //Err(Error::ExpectedVariantColon)
        //}
        unimplemented!("TODO");
    }
}

// `VariantAccess` is provided to the `Visitor` to give it the ability to see
// the content of the single variant that it decided to deserialize.
impl<'de, 'a, 'b> VariantAccess<'de> for Enum<'a, 'de, 'b> {
    type Error = Error;

    // If the `Visitor` expected this variant to be a unit variant, the input
    // should have been the plain string case handled in `deserialize_enum`.
    fn unit_variant(self) -> Result<()> {
        //Err(Error::ExpectedString)
        unimplemented!("TODO");
    }

    // Newtype variants are represented in JSON as `{ NAME: VALUE }` so
    // deserialize the value here.
    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        //seed.deserialize(self.de)
        unimplemented!("TODO");
    }

    // Tuple variants are represented in JSON as `{ NAME: [DATA...] }` so
    // deserialize the sequence of data here.
    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        //de::Deserializer::deserialize_seq(self.de, visitor)
        unimplemented!("TODO");
    }

    // Struct variants are represented in JSON as `{ NAME: { K: V, ... } }` so
    // deserialize the inner map here.
    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        //de::Deserializer::deserialize_map(self.de, visitor)
        unimplemented!("TODO");
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::from_bytes;
    use serde_derive::Deserialize;
    use serde_bytes::ByteBuf;

    #[test]
    fn test_seq() {
        // rust-redbin-helper reduce [-2 299 66666 [5 6] yes 122234.23425 12.5 "aa" "ą" "💖" #"a" #{CAFE}]
        let j = &[0x52, 0x45, 0x44, 0x42, 0x49, 0x4E, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0xAC, 0x00, 0x00, 0x00, 
            0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x00, 0x00, 0x00,
                0x0B, 0x00, 0x00, 0x00, 0xFE, 0xFF, 0xFF, 0xFF,
                0x0B, 0x00, 0x00, 0x00, 0x2B, 0x01, 0x00, 0x00,
                0x0B, 0x00, 0x00, 0x00, 0x6A, 0x04, 0x01, 0x00,
                0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 
                    0x0B, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00,
                    0x0B, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00,
                0x04, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
                0x0C, 0x00, 0x00, 0x00, 0xA3, 0xD7, 0xFD, 0x40, 0x91, 0xED, 0x7C, 0xBF, 0x00, 0x00, 0x00, 0x00,
                0x0C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x29, 0x40, 0x00, 0x00, 0x00, 0x00,
                0x07, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x61, 0x61, 0x00, 0x00,
                0x07, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x05, 0x01, 0x00, 0x00,
                0x07, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x96, 0xF4, 0x01, 0x00,
                0x0A, 0x00, 0x00, 0x00, 0x61, 0x00, 0x00, 0x00,
                0x29, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0xCA, 0xFE, 0x00, 0x00];
        let expected: (i8, i16, u32, Vec<i16>, bool, f64, f32, String, String, String, char, ByteBuf)
            = (-2, 299, 66666, vec![5, 6], true, 122234.23425, 12.5, String::from("aa"), String::from("ą"), String::from("💖"), 'a', ByteBuf::from([0xCA, 0xFE]));
        assert_eq!(expected, from_bytes(j).unwrap());
        
        // rust-redbin-helper none
        assert_eq!((), from_bytes::<()>(
            &[0x52, 0x45, 0x44, 0x42, 0x49, 0x4E, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00,
            0x03, 0x00, 0x00, 0x00]
        ).unwrap());

        // rust-redbin-helper none
        assert_eq!(None, from_bytes::<Option<i32>>(
            &[0x52, 0x45, 0x44, 0x42, 0x49, 0x4E, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00,
            0x03, 0x00, 0x00, 0x00]
        ).unwrap());

        // rust-redbin-helper 123
        assert_eq!(Some(123), from_bytes::<Option<i32>>(
            &[0x52, 0x45, 0x44, 0x42, 0x49, 0x4E, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00,
            0x0B, 0x00, 0x00, 0x00, 0x7B, 0x00, 0x00, 0x00]
        ).unwrap());
        
        
        #[derive(Deserialize, std::cmp::PartialEq, Debug)]
        struct NothingSpecial;
        let test_something = NothingSpecial {};

        // rust-redbin-helper none
        assert_eq!(test_something, from_bytes::<NothingSpecial>(
            &[0x52, 0x45, 0x44, 0x42, 0x49, 0x4E, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00,
            0x03, 0x00, 0x00, 0x00]
        ).unwrap());


        #[derive(Deserialize, std::cmp::PartialEq, Debug)]
        struct Num(i32);
        let fifteen = Num(15);

        // rust-redbin-helper 15
        assert_eq!(fifteen, from_bytes::<Num>(
            &[0x52, 0x45, 0x44, 0x42, 0x49, 0x4E, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00,
            0x0B, 0x00, 0x00, 0x00, 0x0F, 0x00, 0x00, 0x00]
        ).unwrap());


        #[derive(Deserialize, std::cmp::PartialEq, Debug)]
        struct Color(u8, u8, u8);
        let red = Color(255, 0, 0);

        // rust-redbin-helper [255 0 0]
        assert_eq!(red, from_bytes(
            &[0x52, 0x45, 0x44, 0x42, 0x49, 0x4E, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x24, 0x00, 0x00, 0x00,
            0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00,
                0x0B, 0x00, 0x00, 0x00, 0xFF, 0x00, 0x00, 0x00,
                0x0B, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x0B, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
        ).unwrap());


        let mut test_map: std::collections::BTreeMap<String, f64> = std::collections::BTreeMap::new();
        test_map.insert(String::from("a"), 12.5);
        test_map.insert(String::from("b"), 100.1);

        // rust-redbin-helper ["a" 12.5 "b" 100.1]
        assert_eq!(test_map, from_bytes(
            &[0x52, 0x45, 0x44, 0x42, 0x49, 0x4E, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x4C, 0x00, 0x00, 0x00,
            0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00,
                0x07, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x61, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 
                0x0C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x29, 0x40, 0x00, 0x00, 0x00, 0x00,
                0x07, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x62, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 
                0x0C, 0x00, 0x00, 0x00, 0x66, 0x06, 0x59, 0x40, 0x66, 0x66, 0x66, 0x66]
        ).unwrap());


        #[derive(Deserialize, std::cmp::PartialEq, Debug)]
        struct WhatNot { a: f64, b: String }
        let wtf = WhatNot { a: 12.5, b: String::from("sdf") };

        // rust-redbin-helper ["a" 12.5 "b" "sdf"]
        assert_eq!(wtf, from_bytes(
            &[0x52, 0x45, 0x44, 0x42, 0x49, 0x4E, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x50, 0x00, 0x00, 0x00,
            0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00,
                0x07, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x61, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 
                0x0C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x29, 0x40, 0x00, 0x00, 0x00, 0x00,
                0x07, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x62, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 
                0x07, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x73, 0x64, 0x66, 0x00]
        ).unwrap());

    }

}
