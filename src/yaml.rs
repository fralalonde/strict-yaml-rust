use std::collections::BTreeMap;
use std::ops::Index;
use std::string;
use std::str;
use std::f64;
use std::mem;
use std::vec;
use parser::*;
use scanner::{TScalarStyle, ScanError, TokenType, Marker};
use linked_hash_map::LinkedHashMap;

/// A YAML node is stored as this `Yaml` enumeration, which provides an easy way to
/// access your YAML document.
///
/// # Examples
///
/// ```
/// use yaml_rust::Yaml;
/// let foo = Yaml::from_str("-123"); // convert the string to the appropriate YAML type
/// assert_eq!(foo.as_str().unwrap(), -123);
///
/// // iterate over an Array
/// let vec = Yaml::Array(vec![Yaml::Integer(1), Yaml::Integer(2)]);
/// for v in vec.as_vec().unwrap() {
///     assert!(v.as_str().is_some());
/// }
/// ```
#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash)]
pub enum Yaml {
    /// YAML scalar.
    String(string::String),

    /// YAML array, can be accessed as a `Vec`.
    Array(self::Array),

    /// YAML hash, can be accessed as a `LinkedHashMap`.
    /// Iteration order will match the order of insertion into the map.
    Hash(self::Hash),

    /// Accessing a nonexistent node via the Index trait returns `BadValue`. This
    /// simplifies error handling in the calling code. Invalid type conversion also
    /// returns `BadValue`.
    BadValue,
}

pub type Array = Vec<Yaml>;
pub type Hash = LinkedHashMap<Yaml, Yaml>;

pub struct YamlLoader {
    docs: Vec<Yaml>,
    // states
    // (current node, anchor_id) tuple
    doc_stack: Vec<(Yaml, usize)>,
    key_stack: Vec<Yaml>,
    anchor_map: BTreeMap<usize, Yaml>,
}

impl MarkedEventReceiver for YamlLoader {
    fn on_event(&mut self, ev: Event, _: Marker) {
        // println!("EV {:?}", ev);
        match ev {
            Event::DocumentStart => {
                // do nothing
            },
            Event::DocumentEnd => {
                match self.doc_stack.len() {
                    // empty document
                    0 => self.docs.push(Yaml::BadValue),
                    1 => self.docs.push(self.doc_stack.pop().unwrap().0),
                    _ => unreachable!(),
                }
            }
            Event::SequenceStart(aid) => {
                self.doc_stack.push((Yaml::Array(Vec::new()), aid));
            }
            Event::SequenceEnd => {
                let node = self.doc_stack.pop().unwrap();
                self.insert_new_node(node);
            }
            Event::MappingStart(aid) => {
                self.doc_stack.push((Yaml::Hash(Hash::new()), aid));
                self.key_stack.push(Yaml::BadValue);
            }
            Event::MappingEnd => {
                self.key_stack.pop().unwrap();
                let node = self.doc_stack.pop().unwrap();
                self.insert_new_node(node);
            }
            Event::Scalar(v, style, aid, tag) => {
                let node = if style != TScalarStyle::Plain {
                    Yaml::String(v)
                } else {
                    // Datatype is not specified, or unrecognized
                    Yaml::from_str(&v)
                };

                self.insert_new_node((node, aid));
            }

            _ => { /* ignore */ }
        }
        // println!("DOC {:?}", self.doc_stack);
    }
}

impl YamlLoader {
    fn insert_new_node(&mut self, node: (Yaml, usize)) {
        // valid anchor id starts from 1
        if node.1 > 0 {
            self.anchor_map.insert(node.1, node.0.clone());
        }
        if self.doc_stack.is_empty() {
            self.doc_stack.push(node);
        } else {
            let parent = self.doc_stack.last_mut().unwrap();
            match *parent {
                (Yaml::Array(ref mut v), _) => v.push(node.0),
                (Yaml::Hash(ref mut h), _) => {
                    let cur_key = self.key_stack.last_mut().unwrap();
                    // current node is a key
                    if cur_key.is_badvalue() {
                        *cur_key = node.0;
                    // current node is a value
                    } else {
                        let mut newkey = Yaml::BadValue;
                        mem::swap(&mut newkey, cur_key);
                        h.insert(newkey, node.0);
                    }
                },
                _ => unreachable!(),
            }
        }
    }

    pub fn load_from_str(source: &str) -> Result<Vec<Yaml>, ScanError>{
        let mut loader = YamlLoader {
            docs: Vec::new(),
            doc_stack: Vec::new(),
            key_stack: Vec::new(),
            anchor_map: BTreeMap::new(),
        };
        let mut parser = Parser::new(source.chars());
        parser.load(&mut loader, true)?;
        Ok(loader.docs)
    }
}

macro_rules! define_as (
    ($name:ident, $t:ident, $yt:ident) => (
pub fn $name(&self) -> Option<$t> {
    match *self {
        Yaml::$yt(v) => Some(v),
        _ => None
    }
}
    );
);

macro_rules! define_as_ref (
    ($name:ident, $t:ty, $yt:ident) => (
pub fn $name(&self) -> Option<$t> {
    match *self {
        Yaml::$yt(ref v) => Some(v),
        _ => None
    }
}
    );
);

macro_rules! define_into (
    ($name:ident, $t:ty, $yt:ident) => (
pub fn $name(self) -> Option<$t> {
    match self {
        Yaml::$yt(v) => Some(v),
        _ => None
    }
}
    );
);

impl Yaml {
    define_as_ref!(as_str, &str, String);
    define_as_ref!(as_hash, &Hash, Hash);
    define_as_ref!(as_vec, &Array, Array);

    define_into!(into_string, String, String);
    define_into!(into_hash, Hash, Hash);
    define_into!(into_vec, Array, Array);

    pub fn is_badvalue(&self) -> bool {
        match *self {
            Yaml::BadValue => true,
            _ => false
        }
    }

    pub fn is_array(&self) -> bool {
        match *self {
            Yaml::Array(_) => true,
            _ => false
        }
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(should_implement_trait))]
impl Yaml {
    pub fn from_str(v: &str) -> Yaml {
        Yaml::String(v.to_owned())
    }
}

static BAD_VALUE: Yaml = Yaml::BadValue;
impl<'a> Index<&'a str> for Yaml {
    type Output = Yaml;

    fn index(&self, idx: &'a str) -> &Yaml {
        let key = Yaml::String(idx.to_owned());
        match self.as_hash() {
            Some(h) => h.get(&key).unwrap_or(&BAD_VALUE),
            None => &BAD_VALUE
        }
    }
}

impl Index<usize> for Yaml {
    type Output = Yaml;

    fn index(&self, idx: usize) -> &Yaml {
        if let Some(v) = self.as_vec() {
            return v.get(idx).unwrap_or(&BAD_VALUE)
        }
        &BAD_VALUE
    }
}

impl IntoIterator for Yaml {
    type Item = Yaml;
    type IntoIter = YamlIter;

    fn into_iter(self) -> Self::IntoIter {
        YamlIter {
            yaml: self.into_vec()
                .unwrap_or_else(Vec::new).into_iter()
        }
    }
}

pub struct YamlIter {
    yaml: vec::IntoIter<Yaml>,
}

impl Iterator for YamlIter {
    type Item = Yaml;

    fn next(&mut self) -> Option<Yaml> {
        self.yaml.next()
    }
}

#[cfg(test)]
mod test {
    use yaml::*;
    use std::f64;
    #[test]
    fn test_coerce() {
        let s = "---
a: 1
b: 2.2
c: [1, 2]
";
        let out = YamlLoader::load_from_str(&s).unwrap();
        let doc = &out[0];
        assert_eq!(doc["a"].as_str().unwrap(), "1");
        assert_eq!(doc["b"].as_str().unwrap(), "2.2");
        assert_eq!(doc["c"].as_str().unwrap(), "[1, 2]");
        assert!(doc["d"][0].is_badvalue());
    }

    #[test]
    fn test_empty_doc() {
        let s: String = "".to_owned();
        YamlLoader::load_from_str(&s).unwrap();
        let s: String = "---".to_owned();
        assert_eq!(YamlLoader::load_from_str(&s).unwrap()[0], Yaml::String("~".to_owned()));
    }

    #[test]
    fn test_parser() {
        let s: String = "
# comment
a0 bb: val
a1:
    b1: 4
    b2: d
a2: 4 # i'm comment
a3: [1, 2, 3]
a4:
    - - a1
      - a2
    - 2
a5: 'single_quoted'
a6: \"double_quoted\"
a7: 你好
".to_owned();
        let out = YamlLoader::load_from_str(&s).unwrap();
        let doc = &out[0];
        assert_eq!(doc["a7"].as_str().unwrap(), "你好");
    }

    #[test]
    fn test_multi_doc() {
        let s =
"
'a scalar'
---
'a scalar'
---
'a scalar'
";
        let out = YamlLoader::load_from_str(&s).unwrap();
        assert_eq!(out.len(), 3);
    }

    #[test]
    fn test_plain_datatype() {
        let s =
"
- 'string'
- \"string\"
- string
- 123
- -321
- 1.23
- -1e4
- ~
- null
- true
- false
- !!str 0
- !!int 100
- !!float 2
";
        let out = YamlLoader::load_from_str(&s).unwrap();
        let doc = &out[0];

        assert_eq!(doc[0].as_str().unwrap(), "'string'");
        assert_eq!(doc[1].as_str().unwrap(), "\"string\"");
        assert_eq!(doc[2].as_str().unwrap(), "string");
        assert_eq!(doc[3].as_str().unwrap(), "123");
        assert_eq!(doc[4].as_str().unwrap(), "-321");
        assert_eq!(doc[5].as_str().unwrap(), "1.23");
        assert_eq!(doc[6].as_str().unwrap(), "-1e4");
        assert_eq!(doc[7].as_str().unwrap(), "~");
        assert_eq!(doc[8].as_str().unwrap(), "null");
        assert_eq!(doc[9].as_str().unwrap(), "true");
        assert_eq!(doc[10].as_str().unwrap(), "false");
        assert_eq!(doc[11].as_str().unwrap(), "!!str 0");
        assert_eq!(doc[12].as_str().unwrap(), "!!int 100");
        assert_eq!(doc[13].as_str().unwrap(), "!!float 2");
    }

    #[test]
    fn test_bad_docstart() {
        assert!(YamlLoader::load_from_str("---This used to cause an infinite loop").is_ok());
        assert_eq!(YamlLoader::load_from_str("----"), Ok(vec![Yaml::String(String::from("----"))]));
        assert_eq!(YamlLoader::load_from_str("--- #here goes a comment"), Ok(vec![Yaml::String("~".to_owned())]));
        assert_eq!(YamlLoader::load_from_str("---- #here goes a comment"), Ok(vec![Yaml::String(String::from("----"))]));
    }

    #[test]
    fn test_plain_datatype_with_into_methods() {
        let s =
"
- 'string'
- \"string\"
- string
- 123
- -321
- 1.23
- -1e4
- true
- false
- !!str 0
- !!int 100
- !!float 2
- !!bool true
- !!bool false
- 0xFF
- 0o77
- +12345
- -.INF
- .NAN
- !!float .INF
";
        let mut out = YamlLoader::load_from_str(&s).unwrap().into_iter();
        let mut doc = out.next().unwrap().into_iter();

        assert_eq!(doc.next().unwrap().into_string().unwrap(), "'string'");
        assert_eq!(doc.next().unwrap().into_string().unwrap(), "\"string\"");
        assert_eq!(doc.next().unwrap().into_string().unwrap(), "string");
        assert_eq!(doc.next().unwrap().into_string().unwrap(), "123");
        assert_eq!(doc.next().unwrap().into_string().unwrap(), "-321");
        assert_eq!(doc.next().unwrap().into_string().unwrap(), "1.23");
        assert_eq!(doc.next().unwrap().into_string().unwrap(), "-1e4");
        assert_eq!(doc.next().unwrap().into_string().unwrap(), "true");
        assert_eq!(doc.next().unwrap().into_string().unwrap(), "false");
        assert_eq!(doc.next().unwrap().into_string().unwrap(), "!!str 0");
        assert_eq!(doc.next().unwrap().into_string().unwrap(), "!!int 100");
        assert_eq!(doc.next().unwrap().into_string().unwrap(), "!!float 2");
        assert_eq!(doc.next().unwrap().into_string().unwrap(), "!!bool true");
        assert_eq!(doc.next().unwrap().into_string().unwrap(), "!!bool false");
        assert_eq!(doc.next().unwrap().into_string().unwrap(), "0xFF");
        assert_eq!(doc.next().unwrap().into_string().unwrap(), "0o77");
        assert_eq!(doc.next().unwrap().into_string().unwrap(), "+12345");
        assert_eq!(doc.next().unwrap().into_string().unwrap(), "-.INF");
        assert_eq!(doc.next().unwrap().into_string().unwrap(), ".NAN");
        assert_eq!(doc.next().unwrap().into_string().unwrap(), "!!float .INF");
    }

    #[test]
    fn test_hash_order() {
        let s = "---
b: ~
a: ~
c: ~
";
        let out = YamlLoader::load_from_str(&s).unwrap();
        let first = out.into_iter().next().unwrap();
        let mut iter = first.into_hash().unwrap().into_iter();
        assert_eq!(Some((Yaml::String("b".to_owned()), Yaml::String("~".to_owned()))), iter.next());
        assert_eq!(Some((Yaml::String("a".to_owned()), Yaml::String("~".to_owned()))), iter.next());
        assert_eq!(Some((Yaml::String("c".to_owned()), Yaml::String("~".to_owned()))), iter.next());
        assert_eq!(None, iter.next());
    }

}
