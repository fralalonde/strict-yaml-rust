use linked_hash_map::LinkedHashMap;
use parser::*;
use scanner::{Marker, ScanError, TScalarStyle};
use std::error::Error;
use std::fmt;
use std::mem;
use std::ops::Index;
use std::str;
use std::string;
use std::vec;

/// A YAML node is stored as this `Yaml` enumeration, which provides an easy way to
/// access your YAML document.
///
/// # Examples
///
/// ```
/// use strict_yaml_rust::StrictYaml;
/// let foo = StrictYaml::from_str("-123"); // convert the string to the appropriate YAML type
/// assert_eq!(foo.as_str().unwrap(), "-123");
///
/// // iterate over an Array
/// let vec = StrictYaml::Array(vec![StrictYaml::String("1".into()), StrictYaml::String("2".into())]);
/// for v in vec.as_vec().unwrap() {
///     assert!(v.as_str().is_some());
/// }
/// ```
#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash)]
pub enum StrictYaml {
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

#[derive(Clone, PartialEq, Debug, Eq)]
enum StoreError {
    RepeatedHashKey,
}

impl Error for StoreError {}

impl fmt::Display for StoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StoreError::RepeatedHashKey => {
                write!(formatter, "Key already exists in the hash map")
            }
        }
    }
}

pub type Array = Vec<StrictYaml>;
pub type Hash = LinkedHashMap<StrictYaml, StrictYaml>;

pub struct StrictYamlLoader {
    docs: Vec<StrictYaml>,
    // states
    // (current node, anchor_id) tuple
    doc_stack: Vec<(StrictYaml, usize)>,
    key_stack: Vec<StrictYaml>,
}

impl MarkedEventReceiver for StrictYamlLoader {
    fn on_event(&mut self, ev: Event, mark: Marker) -> Result<(), ScanError> {
        // println!("EV {:?}", ev);
        let res = match ev {
            Event::DocumentStart => {
                Ok(())
                // do nothing
            }
            Event::DocumentEnd => {
                match self.doc_stack.len() {
                    // empty document
                    0 => self.docs.push(StrictYaml::BadValue),
                    1 => self.docs.push(self.doc_stack.pop().unwrap().0),
                    _ => unreachable!(),
                }
                Ok(())
            }
            Event::SequenceStart(aid) => {
                self.doc_stack.push((StrictYaml::Array(Vec::new()), aid));
                Ok(())
            }
            Event::SequenceEnd => {
                let node = self.doc_stack.pop().unwrap();
                self.insert_new_node(node)
            }
            Event::MappingStart(aid) => {
                self.doc_stack.push((StrictYaml::Hash(Hash::new()), aid));
                self.key_stack.push(StrictYaml::BadValue);
                Ok(())
            }
            Event::MappingEnd => {
                self.key_stack.pop().unwrap();
                let node = self.doc_stack.pop().unwrap();
                self.insert_new_node(node)
            }
            Event::Scalar(v, style, aid) => {
                let node = if style != TScalarStyle::Plain {
                    StrictYaml::String(v)
                } else {
                    // Datatype is not specified, or unrecognized
                    StrictYaml::from_str(&v)
                };

                self.insert_new_node((node, aid))
            }
            _ => {
                Ok(()) /* ignore */
            }
        };

        res.map_err(|e| ScanError::new(mark, &format!("Error handling node: {}", e)))

        // println!("DOC {:?}", self.doc_stack);
    }
}

impl StrictYamlLoader {
    fn insert_new_node(&mut self, node: (StrictYaml, usize)) -> Result<(), StoreError> {
        // valid anchor id starts from 1
        if self.doc_stack.is_empty() {
            self.doc_stack.push(node);
        } else {
            let parent = self.doc_stack.last_mut().unwrap();
            match *parent {
                (StrictYaml::Array(ref mut v), _) => v.push(node.0),
                (StrictYaml::Hash(ref mut h), _) => {
                    let cur_key = self.key_stack.last_mut().unwrap();

                    // current node is a key
                    if cur_key.is_badvalue() {
                        *cur_key = node.0;
                    // current node is a value
                    } else {
                        let mut newkey = StrictYaml::BadValue;
                        mem::swap(&mut newkey, cur_key);

                        if h.contains_key(&newkey) {
                            return Err(StoreError::RepeatedHashKey);
                        } else {
                            h.insert(newkey, node.0);
                        }
                    }
                }
                _ => unreachable!(),
            }
        }

        Ok(())
    }

    pub fn load_from_str(source: &str) -> Result<Vec<StrictYaml>, ScanError> {
        let mut loader = StrictYamlLoader {
            docs: Vec::new(),
            doc_stack: Vec::new(),
            key_stack: Vec::new(),
        };
        let mut parser = Parser::new(source.chars());
        parser.load(&mut loader, true)?;
        Ok(loader.docs)
    }
}

macro_rules! define_as_ref (
    ($name:ident, $t:ty, $yt:ident) => (
pub fn $name(&self) -> Option<$t> {
    match *self {
       StrictYaml::$yt(ref v) => Some(v),
        _ => None
    }
}
    );
);

macro_rules! define_into (
    ($name:ident, $t:ty, $yt:ident) => (
pub fn $name(self) -> Option<$t> {
    match self {
       StrictYaml::$yt(v) => Some(v),
        _ => None
    }
}
    );
);

impl StrictYaml {
    define_as_ref!(as_str, &str, String);
    define_as_ref!(as_hash, &Hash, Hash);
    define_as_ref!(as_vec, &Array, Array);

    define_into!(into_string, String, String);
    define_into!(into_hash, Hash, Hash);
    define_into!(into_vec, Array, Array);

    pub fn is_badvalue(&self) -> bool {
        matches!(*self, StrictYaml::BadValue)
    }

    pub fn is_array(&self) -> bool {
        matches!(*self, StrictYaml::Array(_))
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(should_implement_trait))]
impl StrictYaml {
    pub fn from_str(v: &str) -> StrictYaml {
        StrictYaml::String(v.to_owned())
    }
}

static BAD_VALUE: StrictYaml = StrictYaml::BadValue;
impl<'a> Index<&'a str> for StrictYaml {
    type Output = StrictYaml;

    fn index(&self, idx: &'a str) -> &StrictYaml {
        let key = StrictYaml::String(idx.to_owned());
        match self.as_hash() {
            Some(h) => h.get(&key).unwrap_or(&BAD_VALUE),
            None => &BAD_VALUE,
        }
    }
}

impl Index<usize> for StrictYaml {
    type Output = StrictYaml;

    fn index(&self, idx: usize) -> &StrictYaml {
        if let Some(v) = self.as_vec() {
            return v.get(idx).unwrap_or(&BAD_VALUE);
        }
        &BAD_VALUE
    }
}

impl IntoIterator for StrictYaml {
    type Item = StrictYaml;
    type IntoIter = YamlIter;

    fn into_iter(self) -> Self::IntoIter {
        YamlIter {
            yaml: self.into_vec().unwrap_or_default().into_iter(),
        }
    }
}

pub struct YamlIter {
    yaml: vec::IntoIter<StrictYaml>,
}

impl Iterator for YamlIter {
    type Item = StrictYaml;

    fn next(&mut self) -> Option<StrictYaml> {
        self.yaml.next()
    }
}

#[cfg(test)]
mod test {
    use strict_yaml::*;
    #[test]
    fn test_coerce() {
        let s = "---
a: 1
b: 2.2
c: [1, 2]
";
        let out = StrictYamlLoader::load_from_str(&s).unwrap();
        let doc = &out[0];
        assert_eq!(doc["a"].as_str().unwrap(), "1");
        assert_eq!(doc["b"].as_str().unwrap(), "2.2");
        assert_eq!(doc["c"].as_str().unwrap(), "[1, 2]");
        assert!(doc["d"][0].is_badvalue());
    }

    #[test]
    fn test_empty_doc() {
        let s: String = "".to_owned();
        StrictYamlLoader::load_from_str(&s).unwrap();
        let s: String = "---".to_owned();
        assert_eq!(
            StrictYamlLoader::load_from_str(&s).unwrap()[0],
            StrictYaml::String("".to_owned())
        );
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
"
        .to_owned();
        let out = StrictYamlLoader::load_from_str(&s).unwrap();
        let doc = &out[0];
        assert_eq!(doc["a7"].as_str().unwrap(), "你好");
    }

    #[test]
    fn test_multi_doc() {
        let s = "
'a scalar'
---
'a scalar'
---
'a scalar'
";
        let out = StrictYamlLoader::load_from_str(&s).unwrap();
        assert_eq!(out.len(), 3);
    }

    #[test]
    fn test_plain_datatype() {
        let s = "
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
- !!null ~
- !!bool true
- !!bool false
- 0xFF
# bad values
- !!int string
- !!float string
- !!bool null
- !!null val
- 0o77
- [ 0xF, 0xF ]
- +12345
- [ true, false ]
";
        let out = StrictYamlLoader::load_from_str(&s).unwrap();
        let doc = &out[0];

        assert_eq!(doc[0].as_str().unwrap(), "string");
        assert_eq!(doc[1].as_str().unwrap(), "string");
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
        assert_eq!(doc[14].as_str().unwrap(), "!!null ~");
        assert_eq!(doc[15].as_str().unwrap(), "!!bool true");
        assert_eq!(doc[16].as_str().unwrap(), "!!bool false");
        assert_eq!(doc[17].as_str().unwrap(), "0xFF");
        assert_eq!(doc[18].as_str().unwrap(), "!!int string");
        assert_eq!(doc[19].as_str().unwrap(), "!!float string");
        assert_eq!(doc[20].as_str().unwrap(), "!!bool null");
        assert_eq!(doc[21].as_str().unwrap(), "!!null val");
        assert_eq!(doc[22].as_str().unwrap(), "0o77");
        assert_eq!(doc[23].as_str().unwrap(), "[ 0xF, 0xF ]");
        assert_eq!(doc[24].as_str().unwrap(), "+12345");
        assert_eq!(doc[25].as_str().unwrap(), "[ true, false ]");
    }

    #[test]
    fn test_bad_docstart() {
        assert!(StrictYamlLoader::load_from_str("---This used to cause an infinite loop").is_ok());
        assert_eq!(
            StrictYamlLoader::load_from_str("----"),
            Ok(vec![StrictYaml::String(String::from("----"))])
        );
        assert_eq!(
            StrictYamlLoader::load_from_str("--- #here goes a comment"),
            Ok(vec![StrictYaml::String("".to_owned())])
        );
        assert_eq!(
            StrictYamlLoader::load_from_str("---- #here goes a comment"),
            Ok(vec![StrictYaml::String(String::from("----"))])
        );
    }

    #[test]
    fn test_plain_datatype_with_into_methods() {
        let s = "
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
        let mut out = StrictYamlLoader::load_from_str(&s).unwrap().into_iter();
        let mut doc = out.next().unwrap().into_iter();

        assert_eq!(doc.next().unwrap().into_string().unwrap(), "string");
        assert_eq!(doc.next().unwrap().into_string().unwrap(), "string");
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
        let out = StrictYamlLoader::load_from_str(&s).unwrap();
        let first = out.into_iter().next().unwrap();
        let mut iter = first.into_hash().unwrap().into_iter();
        assert_eq!(
            Some((
                StrictYaml::String("b".to_owned()),
                StrictYaml::String("~".to_owned())
            )),
            iter.next()
        );
        assert_eq!(
            Some((
                StrictYaml::String("a".to_owned()),
                StrictYaml::String("~".to_owned())
            )),
            iter.next()
        );
        assert_eq!(
            Some((
                StrictYaml::String("c".to_owned()),
                StrictYaml::String("~".to_owned())
            )),
            iter.next()
        );
        assert_eq!(None, iter.next());
    }

    #[test]
    fn test_duplicate_keys() {
        let s = "
a: 10
a: 15
";
        let out = StrictYamlLoader::load_from_str(&s);
        assert!(out.is_err());
        //assert_eq!(out.err(), Actual error type);
    }
}
