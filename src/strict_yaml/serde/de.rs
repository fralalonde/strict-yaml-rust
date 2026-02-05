use crate::{
    serde::error::Error,
    strict_yaml::{Hash, StrictYaml},
};
use serde::de::{
    value::StringDeserializer, Deserialize, DeserializeSeed, Deserializer, EnumAccess,
    Error as SerdeError, IntoDeserializer, MapAccess, SeqAccess, Unexpected, VariantAccess,
    Visitor,
};
use std::{fmt, str::FromStr, vec};

macro_rules! deserialize_number {
    { $f:ident, $v:ident, $t:ty, $err:literal } => {
        fn $f<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
        {
            match self {
                Self::String(value) => {
                    let num = <$t>::from_str_radix(&value, 10).map_err(|_| {
                        Self::Error::invalid_value(
                            Unexpected::Str(&value),
                            &$err,
                        )
                    })?;
                    visitor.$v(num)
                }
                Self::Array(_) => Err(Self::Error::invalid_type(
                    Unexpected::Seq,
                    &$err,
                )),
                Self::Hash(_) => Err(Self::Error::invalid_type(
                    Unexpected::Map,
                    &$err,
                )),
                _ => unreachable!(),
            }
        }
    };
}

macro_rules! deserialize_from_str {
    { $f:ident, $v:ident, $t:ty, $err:literal } => {
        fn $f<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
        {
            match self {
                Self::String(value) => {
                    let v = <$t>::from_str(&value).map_err(|_| {
                        Self::Error::invalid_value(
                            Unexpected::Str(&value),
                            &$err,
                        )
                    })?;
                    visitor.$v(v)
                }
                Self::Array(_) => Err(Self::Error::invalid_type(
                    Unexpected::Seq,
                    &$err,
                )),
                Self::Hash(_) => Err(Self::Error::invalid_type(
                    Unexpected::Map,
                    &$err,
                )),
                _ => unreachable!(),
            }
        }
    };
}

impl<'de> serde::de::Deserializer<'de> for StrictYaml {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    deserialize_number!(deserialize_i8, visit_i8, i8, "an 8-bit signed integer");
    deserialize_number!(deserialize_i16, visit_i16, i16, "a 16-bit signed integer");
    deserialize_number!(deserialize_i32, visit_i32, i32, "a 32-bit signed integer");
    deserialize_number!(deserialize_i64, visit_i64, i64, "a 64-bit signed integer");
    deserialize_number!(
        deserialize_i128,
        visit_i128,
        i128,
        "a 128-bit signed integer"
    );
    deserialize_number!(deserialize_u8, visit_u8, u8, "an 8-bit unsigned integer");
    deserialize_number!(deserialize_u16, visit_u16, u16, "a 16-bit unsigned integer");
    deserialize_number!(deserialize_u32, visit_u32, u32, "a 32-bit unsigned integer");
    deserialize_number!(deserialize_u64, visit_u64, u64, "a 64-bit unsigned integer");
    deserialize_number!(
        deserialize_u128,
        visit_u128,
        u128,
        "a 128-bit unsigned integer"
    );
    deserialize_from_str!(
        deserialize_f32,
        visit_f32,
        f32,
        "a 32-bit floating point number"
    );
    deserialize_from_str!(
        deserialize_f64,
        visit_f64,
        f64,
        "a 64-bit floating point number"
    );
    deserialize_from_str!(deserialize_bool, visit_bool, bool, "a boolean");
    deserialize_from_str!(deserialize_char, visit_char, char, "a single character");

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::String(value) => visitor.visit_string(value),
            Self::Array(_) => Err(Self::Error::invalid_type(Unexpected::Seq, &"a string")),
            Self::Hash(_) => Err(Self::Error::invalid_type(Unexpected::Map, &"a string")),
            _ => unreachable!(),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Self::Error::UnsupportedType("bytes"))
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::BadValue => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::String(value) if value.is_empty() => visitor.visit_unit(),
            Self::Array(value) if value.is_empty() => visitor.visit_unit(),
            Self::Hash(value) if value.is_empty() => visitor.visit_unit(),
            Self::BadValue => visitor.visit_unit(),
            _ => Err(Self::Error::invalid_value(
                Unexpected::Other("non-empty node"),
                &"an empty string, array or map",
            )),
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::String(value) => Err(Self::Error::invalid_type(
                Unexpected::Str(&value),
                &"a sequence",
            )),
            Self::Array(value) => {
                let len = value.len();
                let mut deserializer = ArrayAccess::new(value);
                let seq = visitor.visit_seq(&mut deserializer)?;

                if deserializer.iter.len() != 0 {
                    Err(Self::Error::invalid_length(
                        len,
                        &"fewer elements in sequence",
                    ))
                } else {
                    Ok(seq)
                }
            }
            Self::Hash(_) => Err(Self::Error::invalid_type(Unexpected::Map, &"a sequence")),
            _ => unreachable!(),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
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
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::String(value) => {
                Err(Self::Error::invalid_type(Unexpected::Str(&value), &"a map"))
            }
            Self::Array(_) => Err(Self::Error::invalid_type(Unexpected::Seq, &"a map")),
            Self::Hash(value) => {
                let len = value.len();
                let mut deserializer = HashAccess::new(value);
                let seq = visitor.visit_map(&mut deserializer)?;

                if deserializer.iter.len() != 0 {
                    Err(Self::Error::invalid_length(len, &"fewer elements in map"))
                } else {
                    Ok(seq)
                }
            }
            _ => unreachable!(),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
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
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::String(variant) => visitor.visit_enum(Enum::new(variant, None)),
            Self::Array(_) => Err(Self::Error::invalid_type(
                Unexpected::Seq,
                &"a map or a string",
            )),
            Self::Hash(mut hash) => match hash.pop_front() {
                Some((variant, value)) => {
                    visitor.visit_enum(Enum::new(variant.into_string().unwrap(), Some(value)))
                }
                _ => Err(Self::Error::invalid_type(Unexpected::Map, &"an enum")),
            },
            _ => unreachable!(),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }
}

struct Enum {
    variant: String,
    value: Option<StrictYaml>,
}

impl<'de> Enum {
    fn new(variant: String, value: Option<StrictYaml>) -> Self {
        Self { variant, value }
    }
}

impl<'de> EnumAccess<'de> for Enum {
    type Error = Error;
    type Variant = Variant;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let variant =
            seed.deserialize::<StringDeserializer<Self::Error>>(self.variant.into_deserializer())?;
        let visitor = Variant { value: self.value };
        Ok((variant, visitor))
    }
}

struct Variant {
    value: Option<StrictYaml>,
}

impl<'de> VariantAccess<'de> for Variant {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        match self.value {
            Some(value) => Deserialize::deserialize(value),
            None => Ok(()),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value {
            Some(value) => seed.deserialize(value),
            None => Err(Self::Error::invalid_type(
                Unexpected::UnitVariant,
                &"newtype variant",
            )),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(v) => match v {
                StrictYaml::Array(a) => visitor.visit_seq(ArrayAccess::new(a)),
                _ => unreachable!(),
            },
            None => Err(Self::Error::invalid_type(
                Unexpected::UnitVariant,
                &"tuple variant",
            )),
        }
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(v) => match v {
                StrictYaml::Hash(h) => visitor.visit_map(HashAccess::new(h)),
                _ => unreachable!(),
            },
            None => Err(Self::Error::invalid_type(
                Unexpected::UnitVariant,
                &"struct variant",
            )),
        }
    }
}

struct ArrayAccess {
    iter: vec::IntoIter<StrictYaml>,
}

impl ArrayAccess {
    fn new(v: Vec<StrictYaml>) -> Self {
        Self {
            iter: v.into_iter(),
        }
    }
}

impl<'de> SeqAccess<'de> for ArrayAccess {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(value).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

struct HashAccess {
    iter: <Hash as IntoIterator>::IntoIter,
    value: Option<StrictYaml>,
}

impl HashAccess {
    fn new(map: Hash) -> Self {
        Self {
            iter: map.into_iter(),
            value: None,
        }
    }
}

impl<'de> MapAccess<'de> for HashAccess {
    type Error = Error;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                seed.deserialize(key).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(value) => seed.deserialize(value),
            None => Err(Self::Error::custom("value is missing")),
        }
    }
}

impl<'de> Deserialize<'de> for StrictYaml {
    fn deserialize<D>(deserializer: D) -> Result<StrictYaml, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StrictYamlVisitor;

        impl<'de> Visitor<'de> for StrictYamlVisitor {
            type Value = StrictYaml;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("any valid StrictYaml value")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_string(String::from(value))
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
                Ok(Self::Value::String(value))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut vec = Vec::new();

                while let Some(elem) = seq.next_element()? {
                    vec.push(elem);
                }

                Ok(Self::Value::Array(vec))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut hash = Hash::with_capacity(map.size_hint().unwrap_or_default());

                while let Some((key, value)) = map.next_entry()? {
                    hash.insert(key, value);
                }

                Ok(Self::Value::Hash(hash))
            }
        }

        deserializer.deserialize_any(StrictYamlVisitor)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        serde::{from_str, from_str_many, from_strict_yaml},
        strict_yaml::{Array, Hash, StrictYaml},
    };
    use serde::Deserialize;

    #[test]
    fn test_primitives() {
        let input = StrictYaml::String("true".into());
        assert_eq!(true, from_strict_yaml::<bool>(input).unwrap());

        let input = StrictYaml::String("false".into());
        assert_eq!(false, from_strict_yaml::<bool>(input).unwrap());

        let input = StrictYaml::String("foobar".into());
        assert_eq!(
            "foobar".to_string(),
            from_strict_yaml::<String>(input).unwrap()
        );

        let input = StrictYaml::String("100".into());
        assert_eq!(100, from_strict_yaml::<u16>(input).unwrap());

        let input = StrictYaml::String("-100".into());
        assert_eq!(-100, from_strict_yaml::<i16>(input).unwrap());

        let input = StrictYaml::String("100".into());
        assert_eq!(100.0, from_strict_yaml::<f32>(input).unwrap());

        let input = StrictYaml::String("-100.0".into());
        assert_eq!(-100.0, from_strict_yaml::<f32>(input).unwrap());
    }

    #[test]
    fn test_option() {
        let input = StrictYaml::String("foobar".into());
        assert_eq!(
            Some("foobar".to_string()),
            from_strict_yaml::<Option<String>>(input).unwrap()
        );
    }

    #[test]
    fn test_unit() {
        let input = StrictYaml::String(String::new());
        assert_eq!((), from_strict_yaml::<()>(input).unwrap());

        let input = StrictYaml::Array(Array::new());
        assert_eq!((), from_strict_yaml::<()>(input).unwrap());

        let input = StrictYaml::Hash(Hash::new());
        assert_eq!((), from_strict_yaml::<()>(input).unwrap());
    }

    #[test]
    fn test_unit_struct() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Test;

        let input = StrictYaml::String(String::new());

        assert_eq!(Test, from_strict_yaml::<Test>(input).unwrap());
    }

    #[test]
    fn test_newtype_struct() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Test(String);

        let input = StrictYaml::String("foobar".into());

        assert_eq!(
            Test("foobar".to_string()),
            from_strict_yaml::<Test>(input).unwrap()
        );
    }

    #[test]
    fn test_seq() {
        let input = StrictYaml::Array(vec![
            StrictYaml::String("10".into()),
            StrictYaml::String("20".into()),
            StrictYaml::String("30".into()),
        ]);

        assert_eq!(
            vec![10, 20, 30],
            from_strict_yaml::<Vec<u16>>(input).unwrap()
        );
    }

    #[test]
    fn test_tuple() {
        let input = StrictYaml::Array(vec![
            StrictYaml::String("10".into()),
            StrictYaml::String("20".into()),
            StrictYaml::String("30".into()),
        ]);

        assert_eq!(
            (10, 20, 30),
            from_strict_yaml::<(u16, u16, u16)>(input).unwrap()
        );
    }

    #[test]
    fn test_tuple_struct() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Test(String, bool, u8);

        let input = StrictYaml::Array(vec![
            StrictYaml::String("foobar".into()),
            StrictYaml::String("true".into()),
            StrictYaml::String("20".into()),
        ]);

        assert_eq!(
            Test("foobar".to_string(), true, 20),
            from_strict_yaml::<Test>(input).unwrap()
        );
    }

    #[test]
    fn test_struct() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Test {
            a: bool,
            b: String,
            c: i64,
            d: f64,
        }

        let mut hash = Hash::new();

        hash.insert(
            StrictYaml::String("a".into()),
            StrictYaml::String("true".into()),
        );

        hash.insert(
            StrictYaml::String("c".into()),
            StrictYaml::String("10".into()),
        );

        hash.insert(
            StrictYaml::String("d".into()),
            StrictYaml::String("10.0".into()),
        );

        hash.insert(
            StrictYaml::String("b".into()),
            StrictYaml::String("foobar".into()),
        );

        let input = StrictYaml::Hash(hash);

        let expected = Test {
            a: true,
            b: "foobar".to_string(),
            c: 10,
            d: 10.0,
        };

        assert_eq!(expected, from_strict_yaml::<Test>(input).unwrap());
    }

    #[test]
    fn test_deserialize_any_from_str() {
        let input = r#"
---
foobar
"#;

        assert_eq!(
            StrictYaml::String("foobar".into()),
            from_str::<StrictYaml>(input).unwrap()
        );

        let input = r#"
---
- foo
- bar
- baz
...
"#;

        assert_eq!(
            StrictYaml::Array(vec![
                StrictYaml::String("foo".into()),
                StrictYaml::String("bar".into()),
                StrictYaml::String("baz".into()),
            ]),
            from_str::<StrictYaml>(input).unwrap()
        );

        let input = r#"
---
foo: bar
10: 20
true: false
...
"#;

        let mut hash = Hash::new();
        hash.insert(
            StrictYaml::String("foo".into()),
            StrictYaml::String("bar".into()),
        );
        hash.insert(
            StrictYaml::String("10".into()),
            StrictYaml::String("20".into()),
        );
        hash.insert(
            StrictYaml::String("true".into()),
            StrictYaml::String("false".into()),
        );

        assert_eq!(
            StrictYaml::Hash(hash),
            from_str::<StrictYaml>(input).unwrap()
        );
    }

    #[test]
    fn test_deserialize_any_from_str_many() {
        let input = r#"
---
foo
...
---
bar
---
baz
...
"#;

        assert_eq!(
            vec![
                StrictYaml::String("foo".into()),
                StrictYaml::String("bar".into()),
                StrictYaml::String("baz".into())
            ],
            from_str_many::<Vec<StrictYaml>>(input).unwrap()
        );
    }
}
