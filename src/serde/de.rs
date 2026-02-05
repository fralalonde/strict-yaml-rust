use crate::{
    parser::{Event, Parser},
    serde::error::Error,
    strict_yaml::StrictYaml,
};
use serde::de::{
    Deserialize, DeserializeSeed, EnumAccess, Error as SerdeError, IntoDeserializer, MapAccess,
    SeqAccess, Unexpected, VariantAccess, Visitor,
};
use std::str::{Chars, FromStr};

/// Deserialize an instance of type `T` from [`StrictYaml`](enum@crate::StrictYaml).
///
/// ```
/// use strict_yaml_rust::{StrictYaml, serde::from_strict_yaml};
///
/// let yaml = StrictYaml::Array(
///     vec![
///         StrictYaml::String("1".into()),
///         StrictYaml::String("2".into()),
///         StrictYaml::String("3".into())
///     ]
/// );
///
/// assert_eq!(vec![1, 2, 3], from_strict_yaml::<Vec<u16>>(yaml).unwrap());
/// ```
pub fn from_strict_yaml<'a, T>(yaml: StrictYaml) -> Result<T, Error>
where
    T: Deserialize<'a>,
{
    T::deserialize(yaml)
}

/// A deserializer for the StrictYAML document format.
pub struct Deserializer<'de> {
    parser: Parser<Chars<'de>>,
    // This is used to toggle and keep track of multi-document
    // deserialization. This is only relevant when deserializing a
    // sequence because only sequence types can be deserialized
    // from a StrictYAML stream containing multiple documents.
    is_root: Option<bool>,
}

impl<'de> Deserializer<'de> {
    fn new(input: &'de str, is_root: Option<bool>) -> Self {
        Deserializer {
            parser: Parser::new(input.chars()),
            is_root,
        }
    }

    /// See [`from_str_many`](function@crate::serde::from_str_many) for usage
    /// examples.
    pub fn from_str_many(input: &'de str) -> Self {
        Deserializer::new(input, Some(false))
    }

    /// See [`from_str`](function@crate::serde::from_str) for usage examples.
    pub fn from_str(input: &'de str) -> Self {
        Deserializer::new(input, None)
    }
}

/// Deserialize multiple StrictYAML documents from the same stream into an
/// instance of a container `T`.
///
/// The function serves as a hint to the deserializer to expect a
/// multi-document StrictYAML stream and process it accordingly. The hint from
/// the user is necessary because StrictYAML is not self-describing to the extent
/// that JSON is when it comes to mapping StrictYaml to the `serde` data model.
/// The deserializer needs a way to tell the difference between the
/// following cases when it calls [`serde::Deserializer::deserialize_seq`]:
///
/// ```yaml
/// ---
/// some: example
/// data: 100
/// ---
/// some: example
/// data: 200
/// ---
/// some: example
/// data: 300
/// ```
///
/// ```yaml
/// ---
/// - some: example
///   data: 100
/// - some: example
///   data: 200
/// - some: example
///   data: 300
/// ```
///
/// [`from_str_many`] handles the former case while the latter is the "default
/// mode" when calling [`from_str`].
///
/// # Examples
///
/// As described above the [`from_str_many`] function deserializes a YAML stream containing
/// multiple documents to a container data structure that implements
/// [`serde::Deserialize`] (such as [`Vec`] or [`VecDeque`](struct@std::collections::VecDeque)).
///
///
/// ```rust
/// use strict_yaml_rust::serde::from_str_many;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Deployment {
///     kind: String,
///     spec: Spec
/// }
///
/// #[derive(Deserialize)]
/// struct Spec {
///     replicas: u16,
///     name: String
/// }
///
/// let yaml = r#"
/// ---
/// kind: deployment
/// spec:
///   replicas: 5
///   name: "nginx"
/// ---
/// kind: container
/// spec:
///   replicas: 1
///   name: "redis"
/// ---
/// kind: deployment
/// spec:
///   replicas: 3
///   name: "webapp"
/// ...
/// "#;
///
/// let deployments: Vec<Deployment> = from_str_many(yaml).unwrap();
///
/// assert!(deployments.len() == 3);
/// assert!(deployments.first().is_some_and(|d| d.spec.name == "nginx".to_string()));
/// ```
pub fn from_str_many<'a, T>(s: &'a str) -> Result<T, Error>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_str_many(s);

    // Unlike `from_str` this function skips matching the first
    // events (`StreamStart` and `DocumentStart`) as well as their
    // respective ends, because in this case handling multi-document
    // deserialization is shifted to the `deserialize_seq` method instead.

    T::deserialize(&mut deserializer)
}

/// Deserialize an instance of type `T` from a StrictYAML document.
///
/// # Examples
///
/// The [`from_str`] function deserializes a data structure that
/// implements [`serde::Deserialize`] from a StrictYAML document.
///
/// ```rust
/// use strict_yaml_rust::serde::from_str;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Deployment {
///     kind: String,
///     spec: Spec
/// }
///
/// #[derive(Deserialize)]
/// struct Spec {
///     replicas: u16,
///     name: String
/// }
///
/// let yaml = r#"
/// ---
/// kind: deployment
/// spec:
///   replicas: 5
///   name: nginx
/// ...
/// "#;
///
/// let deployment: Deployment = from_str(yaml).unwrap();
///
/// assert_eq!(deployment.spec.name, "nginx".to_string());
/// ```
pub fn from_str<'a, T>(s: &'a str) -> Result<T, Error>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_str(s);

    // This function expects only a single document inside a
    // stream. Thus it tries to match the expected end events
    // from the loader after deserializing the data "in between".

    let (ev, mark) = deserializer.parser.next()?;

    if ev != Event::StreamStart {
        return Err(Error::from_event(ev, mark, "the start of the stream"));
    }

    let (ev, _mark) = deserializer.parser.peek()?;

    if *ev == Event::DocumentStart {
        deserializer.parser.next()?;
    }

    let res = T::deserialize(&mut deserializer)?;

    let (ev, _mark) = deserializer.parser.peek()?;

    if *ev == Event::DocumentEnd {
        deserializer.parser.next()?;
    }

    let (ev, mark) = deserializer.parser.next()?;

    if ev != Event::StreamEnd {
        return Err(Error::from_event(ev, mark, "the end of the stream"));
    }

    Ok(res)
}

impl<'de> serde::de::Deserializer<'de> for &mut Deserializer<'de> {
    type Error = Error;

    /// This is only called by custom visitor implementations when
    /// the target type does not neatly fit into the serde data model.
    ///
    /// The `StrictYaml` enum type calls into this method to deserialize
    /// efficiently by passing a custom visitor, since we know how
    /// the serde data model types fit into the different variants.
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, mark) = self.parser.peek()?;

        match ev {
            Event::Scalar(_, _, _) => self.deserialize_str(visitor),
            Event::SequenceStart(_) => self.deserialize_seq(visitor),
            Event::MappingStart(_) => self.deserialize_map(visitor),
            _ => {
                return Err(Error::from_event(
                    ev.clone(),
                    mark.clone(),
                    "a sequence, map or scalar",
                ))
            }
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, mark) = self.parser.next()?;

        match ev {
            Event::Scalar(value, _style, _anchor_id) => {
                let boolean = bool::from_str(&value)
                    .map_err(|_| Error::invalid_value(Unexpected::Str(&value), &"a boolean"))
                    .map_err(|err| err + mark)?;

                visitor.visit_bool(boolean).map_err(|err: Error| err + mark)
            }
            _ => Err(Error::from_event(ev, mark, "a scalar")),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, mark) = self.parser.next()?;

        match ev {
            Event::Scalar(value, _style, _anchor_id) => {
                let num = i8::from_str_radix(&value, 10)
                    .map_err(|_| {
                        Error::invalid_value(Unexpected::Str(&value), &"an 8-bit signed integer")
                    })
                    .map_err(|err| err + mark)?;

                visitor.visit_i8(num).map_err(|err: Error| err + mark)
            }
            _ => Err(Error::from_event(ev, mark, "a scalar")),
        }
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, mark) = self.parser.next()?;

        match ev {
            Event::Scalar(value, _style, _anchor_id) => {
                let num = i16::from_str_radix(&value, 10)
                    .map_err(|_| {
                        Error::invalid_value(Unexpected::Str(&value), &"a 16-bit signed integer")
                    })
                    .map_err(|err| err + mark)?;

                visitor.visit_i16(num).map_err(|err: Error| err + mark)
            }
            _ => Err(Error::from_event(ev, mark, "a scalar")),
        }
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, mark) = self.parser.next()?;

        match ev {
            Event::Scalar(value, _style, _anchor_id) => {
                let num = i32::from_str_radix(&value, 10)
                    .map_err(|_| {
                        Error::invalid_value(Unexpected::Str(&value), &"a 32-bit signed integer")
                    })
                    .map_err(|err| err + mark)?;

                visitor.visit_i32(num).map_err(|err: Error| err + mark)
            }
            _ => Err(Error::from_event(ev, mark, "a scalar")),
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, mark) = self.parser.next()?;

        match ev {
            Event::Scalar(value, _style, _anchor_id) => {
                let num = i64::from_str_radix(&value, 10)
                    .map_err(|_| {
                        Error::invalid_value(Unexpected::Str(&value), &"a 64-bit signed integer")
                    })
                    .map_err(|err| err + mark)?;

                visitor.visit_i64(num).map_err(|err: Error| err + mark)
            }
            _ => Err(Error::from_event(ev, mark, "a scalar")),
        }
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, mark) = self.parser.next()?;

        match ev {
            Event::Scalar(value, _style, _anchor_id) => {
                let num = i128::from_str_radix(&value, 10)
                    .map_err(|_| {
                        Error::invalid_value(Unexpected::Str(&value), &"an 128-bit signed integer")
                    })
                    .map_err(|err| err + mark)?;

                visitor.visit_i128(num).map_err(|err: Error| err + mark)
            }
            _ => Err(Error::from_event(ev, mark, "a scalar")),
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, mark) = self.parser.next()?;

        match ev {
            Event::Scalar(value, _style, _anchor_id) => {
                let num = u8::from_str_radix(&value, 10)
                    .map_err(|_| {
                        Error::invalid_value(Unexpected::Str(&value), &"an 8-bit unsigned integer")
                    })
                    .map_err(|err| err + mark)?;

                visitor.visit_u8(num).map_err(|err: Error| err + mark)
            }
            _ => Err(Error::from_event(ev, mark, "a scalar")),
        }
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, mark) = self.parser.next()?;

        match ev {
            Event::Scalar(value, _style, _anchor_id) => {
                let num = u16::from_str_radix(&value, 10)
                    .map_err(|_| {
                        Error::invalid_value(Unexpected::Str(&value), &"a 16-bit unsigned integer")
                    })
                    .map_err(|err| err + mark)?;

                visitor.visit_u16(num).map_err(|err: Error| err + mark)
            }
            _ => Err(Error::from_event(ev, mark, "a scalar")),
        }
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, mark) = self.parser.next()?;

        match ev {
            Event::Scalar(value, _style, _anchor_id) => {
                let num = u32::from_str_radix(&value, 10)
                    .map_err(|_| {
                        Error::invalid_value(Unexpected::Str(&value), &"a 32-bit unsigned integer")
                    })
                    .map_err(|err| err + mark)?;

                visitor.visit_u32(num).map_err(|err: Error| err + mark)
            }
            _ => Err(Error::from_event(ev, mark, "a scalar")),
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, mark) = self.parser.next()?;

        match ev {
            Event::Scalar(value, _style, _anchor_id) => {
                let num = u64::from_str_radix(&value, 10)
                    .map_err(|_| {
                        Error::invalid_value(Unexpected::Str(&value), &"a 64-bit unsigned integer")
                    })
                    .map_err(|err| err + mark)?;

                visitor.visit_u64(num).map_err(|err: Error| err + mark)
            }
            _ => Err(Error::from_event(ev, mark, "a scalar")),
        }
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, mark) = self.parser.next()?;

        match ev {
            Event::Scalar(value, _style, _anchor_id) => {
                let num = u128::from_str_radix(&value, 10)
                    .map_err(|_| {
                        Error::invalid_value(
                            Unexpected::Str(&value),
                            &"an 128-bit unsigned integer",
                        )
                    })
                    .map_err(|err| err + mark)?;

                visitor.visit_u128(num).map_err(|err: Error| err + mark)
            }
            _ => Err(Error::from_event(ev, mark, "a scalar")),
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, mark) = self.parser.next()?;

        match ev {
            Event::Scalar(value, _style, _anchor_id) => {
                let num = f32::from_str(&value)
                    .map_err(|_| {
                        Error::invalid_value(
                            Unexpected::Str(&value),
                            &"a 32-bit floating point number",
                        )
                    })
                    .map_err(|err| err + mark)?;

                visitor.visit_f32(num).map_err(|err: Error| err + mark)
            }
            _ => Err(Error::from_event(ev, mark, "a scalar")),
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, mark) = self.parser.next()?;

        match ev {
            Event::Scalar(value, _style, _anchor_id) => {
                let num = f64::from_str(&value)
                    .map_err(|_| {
                        Error::invalid_value(
                            Unexpected::Str(&value),
                            &"a 64-bit floating point number",
                        )
                    })
                    .map_err(|err| err + mark)?;

                visitor.visit_f64(num).map_err(|err: Error| err + mark)
            }
            _ => Err(Error::from_event(ev, mark, "a scalar")),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, mark) = self.parser.next()?;

        match ev {
            Event::Scalar(value, _style, _anchor_id) => {
                let c = char::from_str(&value)
                    .map_err(|_| {
                        Error::invalid_value(Unexpected::Str(&value), &"a single character")
                    })
                    .map_err(|err| err + mark)?;

                visitor.visit_char(c).map_err(|err: Error| err + mark)
            }
            _ => Err(Error::from_event(ev, mark, "a scalar")),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, mark) = self.parser.next()?;

        match ev {
            Event::Scalar(value, _style, _anchor_id) => {
                visitor.visit_string(value).map_err(|err: Error| err + mark)
            }
            _ => Err(Error::from_event(ev, mark, "a scalar")),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::UnsupportedType("bytes"))
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, _mark) = self.parser.peek()?;

        let is_some = match ev {
            Event::MappingStart(_) | Event::SequenceStart(_) | Event::Scalar(_, _, _) => true,
            _ => false,
        };

        if is_some {
            visitor.visit_some(self)
        } else {
            visitor.visit_none()
        }
    }

    /// Allow deserializing the unit type from empty scalars.
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, _mark) = self.parser.peek()?;

        if matches!(ev, Event::Scalar(value, _style, _anchor_id) if value.is_empty()) {
            let _ = self.parser.next()?;
        }

        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    /// This method is somewhat special since it needs to handle
    /// two deserialization modes:
    /// - deserializing a regular StrictYAML array
    /// - deserializing a multi-document StrictYAML stream
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        // This is the expected value when multi-document deserialization
        // is requested, which is driven by the first call into `deserialize_seq`.
        if self.is_root == Some(false) {
            self.is_root = Some(true);
        }

        let (ev, mark) = self.parser.next()?;

        // Match the next event differently depending on the
        // toggled deserialization mode.
        // `ArrayAccess` is responsible for disabling this mode
        // for nested calls into `deserialize_seq` to avoid toggling
        // unintended behaviour.
        let value = match ev {
            Event::SequenceStart(_) => visitor
                .visit_seq(ArrayAccess::new(self))
                .map_err(|err: Error| err + mark)?,
            Event::StreamStart if self.is_root.is_some() => visitor
                .visit_seq(ArrayAccess::new(self))
                .map_err(|err: Error| err + mark)?,
            _ => {
                if self.is_root.is_some() {
                    return Err(Error::from_event(ev, mark, "the start of the stream"));
                } else {
                    return Err(Error::from_event(ev, mark, "the start of a sequence"));
                }
            }
        };

        if self.is_root == Some(true) {
            let (ev, mark) = self.parser.next()?;

            if ev != Event::StreamEnd {
                return Err(Error::from_event(ev, mark, "the end of the stream"));
            }
        } else {
            let (ev, mark) = self.parser.next()?;

            if ev != Event::SequenceEnd {
                return Err(Error::from_event(ev, mark, "the end of a sequence"));
            }
        }

        Ok(value)
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Error>
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
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, mark) = self.parser.next()?;

        let value = match ev {
            Event::MappingStart(_) => visitor
                .visit_map(HashAccess::new(self))
                .map_err(|err: Error| err + mark)?,
            _ => return Err(Error::from_event(ev, mark, "the start of a mapping")),
        };

        let (ev, _mark) = self.parser.next()?;

        if ev != Event::MappingEnd {
            Err(Error::from_event(ev, mark, "the end of a mapping"))
        } else {
            Ok(value)
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
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
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, mark) = self.parser.next()?;

        match ev {
            Event::Scalar(value, _, _) => visitor
                .visit_enum(value.into_deserializer())
                .map_err(|err: Error| err + mark),
            _ => {
                let v = visitor
                    .visit_enum(Enum::new(self))
                    .map_err(|err: Error| err + mark)?;

                let (ev, mark) = self.parser.next()?;

                if ev != Event::MappingEnd {
                    Err(Error::from_event(ev, mark, "the end of a mapping"))
                } else {
                    Ok(v)
                }
            }
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

struct ArrayAccess<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> ArrayAccess<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Self { de }
    }
}

impl<'de, 'a> SeqAccess<'de> for ArrayAccess<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
    where
        T: DeserializeSeed<'de>,
    {
        let (ev, mark) = self.de.parser.peek()?;

        // When this instance of `ArrayAccess` is responsible for
        // deserializing multiple documents, it needs to temporarily
        // disable this "mode" for nested calls into `deserialize_*`
        // methods. Otherwise a nested `deserialize_seq` would inherit
        // this behaviour, but it needs to be scoped to the top-level
        // sequence.
        if self.de.is_root == Some(true) {
            let mut old_is_root = self.de.is_root.take();

            let res = match ev {
                Event::StreamEnd => Ok(None),
                Event::DocumentStart => {
                    let _ = self.de.parser.next()?;

                    let v = seed.deserialize(&mut *self.de).map(Some)?;

                    let (ev, _mark) = self.de.parser.peek()?;

                    if *ev == Event::DocumentEnd {
                        let _ = self.de.parser.next()?;
                    }

                    Ok(v)
                }
                _ => Err(Error::from_event(
                    ev.clone(),
                    *mark,
                    "the start of a document or the end of the stream",
                )),
            };

            // Reset to the old value. It does not need to be set to
            // `None`, although it would make no difference to process.
            self.de.is_root = old_is_root.take();

            Ok(res?)
        } else {
            match ev {
                Event::SequenceEnd => Ok(None),
                _ => seed.deserialize(&mut *self.de).map(Some),
            }
        }
    }
}

struct HashAccess<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> HashAccess<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Self { de }
    }
}

impl<'de, 'a> MapAccess<'de> for HashAccess<'a, 'de> {
    type Error = Error;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
    where
        T: DeserializeSeed<'de>,
    {
        let (ev, _mark) = self.de.parser.peek()?;

        match ev {
            Event::MappingEnd => Ok(None),
            _ => seed.deserialize(&mut *self.de).map(Some),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Error>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }
}

struct Enum<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> Enum<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Self { de }
    }
}

impl<'de, 'a> EnumAccess<'de> for Enum<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let v = seed.deserialize(&mut *self.de)?;
        Ok((v, self))
    }
}

impl<'de, 'a> VariantAccess<'de> for Enum<'a, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        serde::de::Deserializer::deserialize_seq(self.de, visitor)
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        serde::de::Deserializer::deserialize_map(self.de, visitor)
    }
}
