use crate::error::{Error, Result};
use serde::ser::{self, Serialize};

mod types {
    pub const INTEGER: i32 = 11_i32;
}

pub struct Serializer {
    output: Vec<u8>,
}

pub fn to_bytes<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    let mut header = Vec::from([0x52, 0x45, 0x44, 0x42, 0x49, 0x4E, // "REDBIN"
            0x02, // version
            0x00, // flags
            0x01, 0x00, 0x00, 0x00]); // length (number of records))

    let mut serializer = Serializer {
        output: Vec::new(),
    };
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
        println!("i32: {:?}", v.to_le_bytes());
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
        unimplemented!("TODO");
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        unimplemented!("TODO");
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
        unimplemented!("TODO");
    }

    fn end(self) -> Result<()> {
        unimplemented!("TODO");
    }
}

impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!("TODO");
    }

    fn end(self) -> Result<()> {
        unimplemented!("TODO");
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

#[cfg(test)]
mod tests {
    use super::to_bytes;
    use serde_derive::Serialize;
    
    #[test]
    fn test_int() {
        let i = 5_u8;
        let expected = vec![0x52, 0x45, 0x44, 0x42, 0x49, 0x4E, // "REDBIN"
            0x02, // version
            0x00, // flags
            0x01, 0x00, 0x00, 0x00,  // length (number of records)
            0x08, 0x00, 0x00, 0x00,  // size of payload

            0x0B, 0x00, 0x00, 0x00,  // header, integer! type = 11 (0x0B)
            0x05, 0x00, 0x00, 0x00]; // value, little endian.
        assert_eq!(to_bytes(&i).unwrap(), expected);
    }

/*
    #[test]
    fn test_struct() {
        #[derive(Serialize)]
        struct Test {
            int: u32,
            seq: Vec<&'static str>,
        }

        let test = Test {
            int: 1,
            seq: vec!["a", "b"],
        };
        let expected = r#"{"int":1,"seq":["a","b"]}"#;
        assert_eq!(to_string(&test).unwrap(), expected);
    }

    #[test]
    fn test_enum() {
        #[derive(Serialize)]
        enum E {
            Unit,
            Newtype(u32),
            Tuple(u32, u32),
            Struct { a: u32 },
        }

        let u = E::Unit;
        let expected = r#""Unit""#;
        assert_eq!(to_string(&u).unwrap(), expected);

        let n = E::Newtype(1);
        let expected = r#"{"Newtype":1}"#;
        assert_eq!(to_string(&n).unwrap(), expected);

        let t = E::Tuple(1, 2);
        let expected = r#"{"Tuple":[1,2]}"#;
        assert_eq!(to_string(&t).unwrap(), expected);

        let s = E::Struct { a: 1 };
        let expected = r#"{"Struct":{"a":1}}"#;
        assert_eq!(to_string(&s).unwrap(), expected);
    }
*/
}
