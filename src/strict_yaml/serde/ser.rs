use crate::{
    serde::error::Error,
    strict_yaml::{Hash, StrictYaml},
};
use serde::ser::{
    self, Serialize, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
    SerializeTuple, SerializeTupleStruct, SerializeTupleVariant,
};

pub struct Serializer;

impl ser::Serializer for Serializer {
    type Ok = StrictYaml;
    type Error = Error;

    type SerializeSeq = SerializeArray;
    type SerializeTuple = SerializeArray;
    type SerializeTupleStruct = SerializeArray;
    type SerializeTupleVariant = SerializeArray;
    type SerializeMap = SerializeHash;
    type SerializeStruct = SerializeHash;
    type SerializeStructVariant = SerializeHash;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(Self::Ok::String(v.to_string()))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(Self::Ok::String(v.to_string()))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(Self::Ok::String(v.to_string()))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(Self::Ok::String(v.to_string()))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(Self::Ok::String(v.to_string()))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(Self::Ok::String(v.to_string()))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(Self::Ok::String(v.to_string()))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(Self::Ok::String(v.to_string()))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(Self::Ok::String(v.to_string()))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(Self::Ok::String(v.to_string()))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(Self::Ok::String(v.to_string()))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(Self::Ok::String(v.to_string()))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(Self::Ok::String(v.to_string()))
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(Self::Ok::String(String::new()))
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_none()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
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
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let array = match len {
            None => Vec::new(),
            Some(len) => Vec::with_capacity(len),
        };

        Ok(SerializeArray { array })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        let hash = match len {
            None => Hash::new(),
            Some(len) => Hash::with_capacity(len),
        };

        Ok(SerializeHash { hash, key: None })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        let key = variant.serialize(self)?;
        Ok(SerializeHash {
            hash: Hash::with_capacity(len),
            key: Some(key),
        })
    }
}

pub struct SerializeArray {
    array: Vec<StrictYaml>,
}

impl SerializeSeq for SerializeArray {
    type Ok = StrictYaml;
    type Error = Error;

    fn serialize_element<T>(&mut self, elem: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        self.array.push(elem.serialize(Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Self::Ok::Array(self.array))
    }
}

impl SerializeTuple for SerializeArray {
    type Ok = StrictYaml;
    type Error = Error;

    fn serialize_element<T>(&mut self, elem: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        self.array.push(elem.serialize(Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Self::Ok::Array(self.array))
    }
}

impl SerializeTupleStruct for SerializeArray {
    type Ok = StrictYaml;
    type Error = Error;

    fn serialize_field<T>(&mut self, elem: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        self.array.push(elem.serialize(Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Self::Ok::Array(self.array))
    }
}

impl SerializeTupleVariant for SerializeArray {
    type Ok = StrictYaml;
    type Error = Error;

    fn serialize_field<T>(&mut self, elem: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        self.array.push(elem.serialize(Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Self::Ok::Array(self.array))
    }
}

pub struct SerializeHash {
    hash: Hash,
    key: Option<StrictYaml>,
}

impl SerializeMap for SerializeHash {
    type Ok = StrictYaml;
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        self.key = Some(key.serialize(Serializer)?);
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        let value = value.serialize(Serializer)?;

        match self.key.take() {
            Some(key) => self.hash.insert(key, value),
            None => {
                return Err(Error::Message(
                    "serialize_value called before serialize_key".to_string(),
                ))
            }
        };

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Self::Ok::Hash(self.hash))
    }
}

impl SerializeStruct for SerializeHash {
    type Ok = StrictYaml;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        let key = key.serialize(Serializer)?;
        let value = value.serialize(Serializer)?;

        self.hash.insert(key, value);

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Self::Ok::Hash(self.hash))
    }
}

impl SerializeStructVariant for SerializeHash {
    type Ok = StrictYaml;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let key = key.serialize(Serializer)?;
        let value = value.serialize(Serializer)?;

        self.hash.insert(key, value);

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Self::Ok::Hash(self.hash))
    }
}

#[cfg(test)]
mod test {
    use crate::{serde::to_strict_yaml, strict_yaml::StrictYaml};
    use serde::Serialize;

    #[test]
    fn test_primitives() {
        let input = "foobar";

        let yaml = StrictYaml::String(input.to_string());

        assert_eq!(to_strict_yaml(input).unwrap(), yaml);

        let input = true;

        let yaml = StrictYaml::String(input.to_string());

        assert_eq!(to_strict_yaml(input).unwrap(), yaml);

        let input = 100;

        let yaml = StrictYaml::String(input.to_string());

        assert_eq!(to_strict_yaml(input).unwrap(), yaml);

        let input = 100.0;

        let yaml = StrictYaml::String(input.to_string());

        assert_eq!(to_strict_yaml(input).unwrap(), yaml);
    }

    #[test]
    fn test_none() {
        assert_eq!(
            to_strict_yaml(None::<Vec<u8>>).unwrap(),
            StrictYaml::String("".into())
        );
    }

    #[test]
    fn test_some() {
        assert_eq!(
            to_strict_yaml(Some(true)).unwrap(),
            StrictYaml::String("true".into())
        );
    }

    #[test]
    fn test_unit() {
        assert_eq!(to_strict_yaml(()).unwrap(), StrictYaml::String("".into()));
    }

    #[test]
    fn test_unit_variant() {
        #[derive(Serialize)]
        enum Test {
            Foobar,
        }

        assert_eq!(
            to_strict_yaml(Test::Foobar).unwrap(),
            StrictYaml::String("Foobar".into())
        );
    }

    #[test]
    fn test_newtype_struct() {
        #[derive(Serialize)]
        struct Test(bool);

        assert_eq!(
            to_strict_yaml(Test(true)).unwrap(),
            StrictYaml::String("true".into())
        );
    }

    #[test]
    fn test_newtype_variant() {
        #[derive(Serialize)]
        enum Test {
            Foobar(bool),
        }

        assert_eq!(
            to_strict_yaml(Test::Foobar(true)).unwrap(),
            StrictYaml::String("true".into())
        );
    }

    #[test]
    fn test_seq() {
        let input = vec![1, 2, 3];

        let yaml = StrictYaml::Array(vec![
            StrictYaml::String("1".into()),
            StrictYaml::String("2".into()),
            StrictYaml::String("3".into()),
        ]);

        assert_eq!(to_strict_yaml(input).unwrap(), yaml);
    }

    #[test]
    fn test_tuple() {
        let input = (1, true, "foobar");

        let yaml = StrictYaml::Array(vec![
            StrictYaml::String("1".into()),
            StrictYaml::String("true".into()),
            StrictYaml::String("foobar".into()),
        ]);

        assert_eq!(to_strict_yaml(input).unwrap(), yaml);
    }

    #[test]
    fn test_tuple_struct() {
        #[derive(Serialize)]
        struct Test(u8, bool, &'static str);

        let input = Test(1, true, "foobar");

        let yaml = StrictYaml::Array(vec![
            StrictYaml::String("1".into()),
            StrictYaml::String("true".into()),
            StrictYaml::String("foobar".into()),
        ]);

        assert_eq!(to_strict_yaml(input).unwrap(), yaml);
    }
}
