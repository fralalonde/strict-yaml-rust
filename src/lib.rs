// Copyright 2015, Yuheng Chen. See the LICENSE file at the top-level
// directory of this distribution.

//! Strict YAML implementation in pure Rust.
//!
//! # Usage
//!
//! This crate is [on github](https://github.com/fralalonde/strict-yaml-rust) and can be
//! used by adding `strict-yaml` to the dependencies in your project's `Cargo.toml`.
//!
//! ```sh
//! cargo add strict-yaml-rust
//! ```
//!
//! # Examples
//!
//! ```
//! use strict_yaml_rust::{StrictYamlLoader, StrictYamlEmitter};
//!
//! let docs = StrictYamlLoader::load_from_str("zug: [1, 2, 3]").unwrap();
//! let doc = &docs[0]; // select the first document
//! assert_eq!(doc["zug"].as_str(), Some("[1, 2, 3]")); // access elements by key
//!
//! let mut out_str = String::new();
//! let mut emitter = StrictYamlEmitter::new(&mut out_str);
//! emitter.dump(doc).unwrap(); // dump the YAML object to a String
//! ```
//!
//! # Serde
//!
//! The crate also provides a [`serde`](https://serde.rs) implementation. The functions
//! in the [`serde`](module@crate::serde) module can be enabled with the `serde` feature flag.

pub mod emitter;
pub mod parser;
pub mod scanner;
#[cfg(feature = "serde")]
pub mod serde;
pub mod strict_yaml;

// reexport key APIs
pub use crate::emitter::{EmitError, StrictYamlEmitter};
pub use crate::parser::Event;
pub use crate::scanner::ScanError;
pub use crate::strict_yaml::{StrictYaml, StrictYamlLoader};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api() {
        let s = "
# from yaml-cpp example
- name: Ogre
  position: [0, 5, 0]
  powers:
    - name: Club
      damage: 10
    - name: Fist
      damage: 8
- name: Dragon
  position: [1, 0, 10]
  powers:
    - name: Fire Breath
      damage: 25
    - name: Claws
      damage: 15
- name: Wizard
  position: [5, -3, 0]
  powers:
    - name: Acid Rain
      damage: 50
    - name: Staff
      damage: 3
";
        let docs = StrictYamlLoader::load_from_str(s).unwrap();
        let doc = &docs[0];

        assert_eq!(doc[0]["name"].as_str().unwrap(), "Ogre");

        let mut writer = String::new();
        {
            let mut emitter = StrictYamlEmitter::new(&mut writer);
            emitter.dump(doc).unwrap();
        }

        assert!(!writer.is_empty());
    }

    fn try_fail(s: &str) -> Result<Vec<StrictYaml>, ScanError> {
        let t = StrictYamlLoader::load_from_str(s)?;
        Ok(t)
    }

    #[test]
    fn test_fail() {
        let s = "
# syntax error
scalar
key: [1, 2]]
key1:a2
";
        assert!(StrictYamlLoader::load_from_str(s).is_err());
        assert!(try_fail(s).is_err());
    }
}
