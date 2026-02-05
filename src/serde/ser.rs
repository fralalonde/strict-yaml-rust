use crate::{
    emitter::{escape_str, need_quotes, StrictYamlEmitter},
    serde::error::Error,
    strict_yaml::{self, StrictYaml},
};
use serde::ser::{
    self, Serialize, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
    SerializeTuple, SerializeTupleStruct, SerializeTupleVariant,
};
use std::fmt;

/// Serialize an instance of type `T` to [`StrictYaml`](enum@crate::StrictYaml).
///
/// ```
/// use strict_yaml_rust::{StrictYaml, serde::to_strict_yaml};
///
/// let v = vec![1, 2, 3];
///
/// let yaml = StrictYaml::Array(
///     vec![
///         StrictYaml::String("1".into()),
///         StrictYaml::String("2".into()),
///         StrictYaml::String("3".into())
///     ]
/// );
///
/// assert_eq!(yaml, to_strict_yaml(v).unwrap());
/// ```
pub fn to_strict_yaml<T: Serialize>(value: T) -> Result<StrictYaml, Error> {
    value.serialize(strict_yaml::serde::ser::Serializer)
}

/// Serialize an instance of type `T` to a StrictYAML document.
///
/// # Examples
///
/// The [`to_string`] function serializes a data structure that
/// implements [`serde::Serialize`] to a StrictYAML document.
///
/// ```rust
/// use strict_yaml_rust::serde::to_string;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Deployment {
///     kind: &'static str,
///     spec: Spec
/// }
///
/// #[derive(Serialize)]
/// struct Spec {
///     replicas: u16,
///     name: &'static str
/// }
///
/// let deployment = Deployment {
///     kind: "deployment",
///     spec: Spec {
///         replicas: 5,
///         name: "nginx"
///     }
/// };
///
/// let output = r#"---
/// kind: deployment
/// spec:
///   replicas: "5"
///   name: nginx
/// "#;
///
/// assert_eq!(output, to_string(&deployment).unwrap());
/// ```
pub fn to_string<T>(value: &T) -> Result<String, Error>
where
    T: Serialize,
{
    let mut out = String::new();

    let mut serializer = Serializer {
        emitter: StrictYamlEmitter::new(&mut out),
        scope: None,
    };

    write!(serializer.emitter.writer, "---")?;
    writeln!(serializer.emitter.writer)?;

    value.serialize(&mut serializer)?;

    Ok(out)
}

/// Serialize a container of type `T` to a StrictYAML document stream.
///
/// Similar to the deserialization function [`from_str_many`](function@crate::serde::from_str_many)
/// this function serves as a hint to the serializer to serialize a
/// container that implements [`serde::Serialize`] (such as [`Vec`] or
/// [`VecDeque`](struct@std::collections::VecDeque) to a StrictYAML document stream.
///
/// In contrast [`to_string`] would serialize such a data structure to a single
/// StrictYAML document containing an array at the root.
///
/// # Examples
///
/// As described above the [`to_string_many`] function serializes a
/// data structure to a StrictYAML document stream containing multiple documents.
///
/// ```rust
/// use strict_yaml_rust::serde::to_string_many;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Deployment {
///     kind: &'static str,
///     spec: Spec
/// }
///
/// #[derive(Serialize)]
/// struct Spec {
///     replicas: u16,
///     name: &'static str
/// }
///
/// let output = r#"---
/// kind: deployment
/// spec:
///   replicas: "5"
///   name: nginx
/// ---
/// kind: container
/// spec:
///   replicas: "1"
///   name: redis
/// ---
/// kind: deployment
/// spec:
///   replicas: "3"
///   name: webapp
/// "#;
///
/// let deployments = vec![
///     Deployment {
///         kind: "deployment",
///         spec: Spec {
///             replicas: 5,
///             name: "nginx"
///         }
///     },
///     Deployment {
///         kind: "container",
///         spec: Spec {
///             replicas: 1,
///             name: "redis"
///         }
///     },
///     Deployment {
///         kind: "deployment",
///         spec: Spec {
///             replicas: 3,
///             name: "webapp"
///         }
///     },
/// ];
///
/// assert_eq!(output, to_string_many(&deployments).unwrap());
/// ```
pub fn to_string_many<T>(value: &T) -> Result<String, Error>
where
    T: Serialize,
{
    let mut out = String::new();

    let mut serializer = Serializer {
        emitter: StrictYamlEmitter::new(&mut out),
        scope: Some(Scope::Root),
    };

    value.serialize(&mut serializer)?;

    Ok(out)
}

#[derive(Debug, PartialEq)]
enum Scope {
    Root,
    Key,
    Map,
    Seq,
}

/// A serializer for the StrictYAML document format.
pub struct Serializer<'a> {
    emitter: StrictYamlEmitter<'a>,
    // This attribute is used to keep track of the parent
    // node type in contexts where the parent node
    // type affects serialization of the currently
    // processed child node.
    // For example a map is serialized differently
    // when it is a value inside another map or an
    // item in a sequence.
    scope: Option<Scope>,
}

/// This function is used to serialize any kind of scalar types that are
/// available in the serde data model.
/// However there is some nuance to different types, e.g. a string might
/// contain newlines which should be serialized as a multi-line YAML string,
/// which also affects indentation.
/// At the same time numeric types are exclusively single-line.
///
/// This function uses the quoting rules from `StrictYamlEmitter`.
///
/// It also writes a newline after serializing a scalar, except when the
/// scalar is a map key, which might be followed by a scalar value on the
/// same line.
fn write_str<T: fmt::Display>(
    serializer: &mut Serializer,
    v: T,
    maybe_multi_line: bool,
) -> Result<(), Error> {
    let s = v.to_string();

    if serializer.scope == Some(Scope::Key) {
        serializer.emitter.writer.write_char(' ')?;
    }

    if maybe_multi_line && s.ends_with('\n') {
        serializer.emitter.writer.write_char('|')?;
        writeln!(serializer.emitter.writer)?;

        // Indentation differs for multi-line strings depending
        // on the context.
        let level_delta = if serializer.emitter.level < 0 || serializer.scope == Some(Scope::Seq) {
            2
        } else {
            1
        };

        serializer.emitter.level += level_delta;

        for line in s.lines() {
            serializer.emitter.write_indent()?;
            serializer.emitter.writer.write_str(line)?;
            writeln!(serializer.emitter.writer)?;
        }

        serializer.emitter.level -= level_delta;
    } else {
        if need_quotes(&s) {
            escape_str(serializer.emitter.writer, &s)?;
        } else {
            serializer.emitter.writer.write_str(&s)?;
        }

        // This scope is set when the scalar is a map key
        // in which case it may be followed by a scalar value
        // on the same line and the newline is skipped.
        if serializer.scope != Some(Scope::Map) {
            writeln!(serializer.emitter.writer)?;
        }
    }

    Ok(())
}

impl ser::Serializer for &'_ mut Serializer<'_> {
    type Ok = ();
    type Error = Error;

    // Since no extra state is needed to keep track of the
    // serialization process, `Serializer` can implement every
    // specified trait.
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        write_str(&mut *self, v, false)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        write_str(&mut *self, v, false)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        write_str(&mut *self, v, false)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        write_str(&mut *self, v, false)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        write_str(&mut *self, v, false)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        write_str(&mut *self, v, false)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        write_str(&mut *self, v, false)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        write_str(&mut *self, v, false)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        write_str(&mut *self, v, false)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        write_str(&mut *self, v, false)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        write_str(&mut *self, v, false)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        write_str(&mut *self, v, false)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        write_str(&mut *self, v, true)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let s = std::str::from_utf8(v)?;

        write_str(&mut *self, s, true)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
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
        variant.serialize(self)
    }

    /// Newtype structs are serialized by serializing the inner
    /// value and ignoring the wrapper.
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

    /// Newtype enums are serialized as maps, i.e. `key: value`, where
    /// `variant` is the key and the value is any type that implements
    /// `Serialize`.
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        // Start with a newline if the newtype enum is a nested value
        // inside a map with a given key.
        if self.scope == Some(Scope::Key) {
            writeln!(self.emitter.writer)?;
        }

        self.emitter.level += 1;

        // Skip indentation if the newtype enum is nested inside a
        // sequence and the key/variant follows a dash `-`.
        if self.scope != Some(Scope::Seq) {
            self.emitter.write_indent()?;
        }

        // Serialize the variant as if it were a key in a regular
        // map.
        let mut old_scope = self.scope.replace(Scope::Map);
        variant.serialize(&mut *self)?;
        self.scope = old_scope.take();

        write!(self.emitter.writer, ":")?;

        // Serialize the value as if it were a value in a regular
        // map following a key.
        old_scope = self.scope.replace(Scope::Key);
        let v = value.serialize(&mut *self)?;
        self.scope = old_scope.take();

        self.emitter.level -= 1;

        Ok(v)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        // Unless the values in the sequence are serialized as
        // multiple documents (as indicated by `self.scope`), empty
        // sequences are serialized inline with empty brackets.
        if self.scope != Some(Scope::Root) {
            if len == Some(0) {
                write!(self.emitter.writer, "[]")?;
            } else {
                self.emitter.level += 1;
            }
        }

        // A newline must come before a sequence when the sequence
        // follows a map key. In contrast a sequence that is nested in
        // another sequence is started on the same line following the dash `-`.
        if self.scope == Some(Scope::Key) {
            writeln!(self.emitter.writer)?;
        }

        Ok(self)
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

    /// Tuple variant enums are serialized as maps, i.e. `key: value`,
    /// where `variant` is the key and the value is a sequence.
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        // Start with a newline if the tuple variant is a nested value
        // inside a map with a given key.
        if self.scope == Some(Scope::Key) {
            writeln!(self.emitter.writer)?;
        }

        self.emitter.level += 1;

        // Skip indentation if the tuple variant is nested inside a
        // sequence and the key/variant follows a dash `-`.
        if self.scope != Some(Scope::Seq) {
            self.emitter.write_indent()?;
        }

        // Serialize the variant as if it were a key in a regular
        // map.
        let mut old_scope = self.scope.replace(Scope::Map);
        variant.serialize(&mut *self)?;
        self.scope = old_scope.take();

        write!(self.emitter.writer, ":")?;

        self.scope.replace(Scope::Key);

        // Serialize the tuple as if it were a value in a regular
        // map following a key.
        self.serialize_seq(Some(len))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        if len == Some(0) {
            write!(self.emitter.writer, "{{}}")?;
        } else {
            self.emitter.level += 1;
        }

        // When the map is nested inside another map the preceding
        // key must first be followed by a newline.
        if self.scope == Some(Scope::Key) {
            writeln!(self.emitter.writer)?;
        }

        Ok(self)
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
        // Start with a newline if the tuple variant is a nested value
        // inside a map with a given key.
        if self.scope == Some(Scope::Key) {
            writeln!(self.emitter.writer)?;
        }

        self.emitter.level += 1;

        // Skip indentation if the tuple variant is nested inside a
        // sequence and the key/variant follows a dash `-`.
        if self.scope != Some(Scope::Seq) {
            self.emitter.write_indent()?;
        }

        // Serialize the variant as if it were a key in a regular
        // map.
        let mut old_scope = self.scope.replace(Scope::Map);
        variant.serialize(&mut *self)?;
        self.scope = old_scope.take();

        write!(self.emitter.writer, ":")?;

        self.scope.replace(Scope::Key);

        // Serialize the value as a nested map following a key from
        // the parent map.
        self.serialize_map(Some(len))
    }
}

impl SerializeSeq for &'_ mut Serializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        // The `Root` scope indicates to the serializer that the sequence
        // is to be serialized as a multi-line StrictYAML stream.
        // As such each element is preceded by document delimiters.
        if self.scope == Some(Scope::Root) {
            write!(self.emitter.writer, "---")?;
            writeln!(self.emitter.writer)?;

            let mut old_scope = self.scope.take();
            value.serialize(&mut **self)?;
            self.scope = old_scope.take();
        } else {
            // Otherwise the sequence is serialized in a regular manner.
            // Indentation is skipped for this element if it is the
            // first element from a sequence nested in another sequence.
            if self.scope != Some(Scope::Seq) {
                self.emitter.write_indent()?;
            } else {
                self.scope = None;
            }

            write!(self.emitter.writer, "- ")?;

            let mut old_scope = self.scope.replace(Scope::Seq);
            value.serialize(&mut **self)?;
            self.scope = old_scope.take();
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.emitter.level -= 1;

        if self.scope == Some(Scope::Root) {
            self.scope = None;
        }

        Ok(())
    }
}

impl SerializeTuple for &'_ mut Serializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        // The `Root` scope indicates to the serializer that the sequence
        // is to be serialized as a multi-line StrictYAML stream.
        // As such each element is preceded by document delimiters.
        if self.scope == Some(Scope::Root) {
            write!(self.emitter.writer, "---")?;
            writeln!(self.emitter.writer)?;

            let mut old_scope = self.scope.take();
            value.serialize(&mut **self)?;
            self.scope = old_scope.take();
        } else {
            // Otherwise the sequence is serialized in a regular manner.
            // Indentation is skipped for this element if it is the
            // first element from a sequence nested in another sequence.
            if self.scope != Some(Scope::Seq) {
                self.emitter.write_indent()?;
            } else {
                self.scope = None;
            }

            write!(self.emitter.writer, "- ")?;

            let mut old_scope = self.scope.replace(Scope::Seq);
            value.serialize(&mut **self)?;
            self.scope = old_scope.take();
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.emitter.level -= 1;

        if self.scope == Some(Scope::Root) {
            self.scope = None;
        }

        Ok(())
    }
}

impl SerializeTupleStruct for &'_ mut Serializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        // The `Root` scope indicates to the serializer that the sequence
        // is to be serialized as a multi-line StrictYAML stream.
        // As such each element is preceded by document delimiters.
        if self.scope == Some(Scope::Root) {
            write!(self.emitter.writer, "---")?;
            writeln!(self.emitter.writer)?;

            let mut old_scope = self.scope.take();
            value.serialize(&mut **self)?;
            self.scope = old_scope.take();
        } else {
            // Otherwise the sequence is serialized in a regular manner.
            // Indentation is skipped for this element if it is the
            // first element from a sequence nested in another sequence.
            if self.scope != Some(Scope::Seq) {
                self.emitter.write_indent()?;
            } else {
                self.scope = None;
            }

            write!(self.emitter.writer, "- ")?;

            let mut old_scope = self.scope.replace(Scope::Seq);
            value.serialize(&mut **self)?;
            self.scope = old_scope.take();
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.emitter.level -= 1;

        if self.scope == Some(Scope::Root) {
            self.scope = None;
        }

        Ok(())
    }
}

impl SerializeTupleVariant for &'_ mut Serializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        // The `Root` scope indicates to the serializer that the sequence
        // is to be serialized as a multi-line StrictYAML stream.
        // As such each element is preceded by document delimiters.
        if self.scope == Some(Scope::Root) {
            write!(self.emitter.writer, "---")?;
            writeln!(self.emitter.writer)?;

            let mut old_scope = self.scope.take();
            value.serialize(&mut **self)?;
            self.scope = old_scope.take();
        } else {
            // Otherwise the sequence is serialized in a regular manner.
            // Indentation is skipped for this element if it is the
            // first element from a sequence nested in another sequence.
            if self.scope != Some(Scope::Seq) {
                self.emitter.write_indent()?;
            } else {
                self.scope = None;
            }

            write!(self.emitter.writer, "- ")?;

            let mut old_scope = self.scope.replace(Scope::Seq);
            value.serialize(&mut **self)?;
            self.scope = old_scope.take();
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.emitter.level -= 1;

        if self.scope == Some(Scope::Root) {
            self.scope = None;
        }

        Ok(())
    }
}

impl SerializeMap for &'_ mut Serializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        // Skip the indentation if this key is the first in a nested map
        // inside a sequence, to follow the dash `-` directly.
        if self.scope != Some(Scope::Seq) {
            self.emitter.write_indent()?;
        } else {
            self.scope = None;
        }

        let mut old_scope = self.scope.replace(Scope::Map);
        key.serialize(&mut **self)?;
        self.scope = old_scope.take();

        write!(self.emitter.writer, ":")?;

        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        let mut old_scope = self.scope.replace(Scope::Key);
        value.serialize(&mut **self)?;
        self.scope = old_scope.take();
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.emitter.level -= 1;
        Ok(())
    }
}

impl SerializeStruct for &'_ mut Serializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        // Skip the indentation if this key is the first in a nested map
        // inside a sequence, to follow the dash `-` directly.
        if self.scope != Some(Scope::Seq) {
            self.emitter.write_indent()?;
        } else {
            self.scope = None;
        }

        let mut old_scope = self.scope.replace(Scope::Map);
        key.serialize(&mut **self)?;
        self.scope = old_scope.take();

        write!(self.emitter.writer, ":")?;

        old_scope = self.scope.replace(Scope::Key);
        value.serialize(&mut **self)?;
        self.scope = old_scope.take();

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.emitter.level -= 1;
        Ok(())
    }
}

impl SerializeStructVariant for &'_ mut Serializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        // Skip the indentation if this key is the first in a nested map
        // inside a sequence, to follow the dash `-` directly.
        if self.scope != Some(Scope::Seq) {
            self.emitter.write_indent()?;
        } else {
            self.scope = None;
        }

        let mut old_scope = self.scope.replace(Scope::Map);
        key.serialize(&mut **self)?;
        self.scope = old_scope.take();

        write!(self.emitter.writer, ":")?;

        old_scope = self.scope.replace(Scope::Key);
        value.serialize(&mut **self)?;
        self.scope = old_scope.take();

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        // After serializing a struct variant the indentation level
        // needs to decrease by two, because the level is increased
        // twice during the process:
        // - before the key (variant) is serialized
        // - after the method calls into `serialize_map` to serialize
        //   its fields
        self.emitter.level -= 2;
        Ok(())
    }
}
