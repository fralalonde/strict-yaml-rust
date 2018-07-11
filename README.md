strict-yaml-rust
-----
A [StrictYAML](http://hitchdev.com/strictyaml/) implementation for Rust 
obtained by savagely chopping off code from [yaml-rust](https://crates.io/crates/yaml-rust).

[![Build Status](https://travis-ci.org/fralalonde/strict-yaml-rust.svg?branch=master)](https://travis-ci.org/fralalonde/strict-yaml-rust)
[![Build status](https://ci.appveyor.com/api/projects/status/scf47535ckp4ylg4?svg=true)](https://ci.appveyor.com/project/fralalonde/strict-yaml-rust)
[![license](https://img.shields.io/crates/l/strict-yaml-rust.svg)](https://crates.io/crates/strict-yaml-rust/)
[![version](https://img.shields.io/crates/v/strict-yaml-rust.svg)](https://crates.io/crates/strict-yaml-rust/)

This crate was originally started as feature-gated (`#[cfg(feature)]`) fork of the original.

Making it standalone allows to use both implementations (full _and_ strict) from the same app, with confidence that the documents 
expected to be StrictYAML compliant will _not_ be parsed as full YAML by mistake.

**Mad props** going to the original crate author, [Chen Yuheng](https://github.com/chyh1990).

## What is StrictYAML?

StrictYAML is a subset of the YAML format, removing troublesome parts of the specification:

 - No typing. StrictYAML only knows Strings, Arrays and Dicts (Maps).  
 - No duplicate keys.
 - No tags.
 - No anchors or refs.
 - No "flow" style (embedded JSON).
 
In short, keeping only the parts of YAML that we all know and love.

For more details, see the original documentation and implementation:

 - http://hitchdev.com/strictyaml/
 - https://github.com/crdoconnor/strictyaml 

## Quick Start

Add the following to the Cargo.toml of your project:

```toml
[dependencies]
strict-yaml-rust = "0.1"
```

or

```toml
[dependencies.yaml-rust]
git = "https://github.com/fralalonde/strict-yaml-rust.git"
```

and import:

```rust
extern crate yaml_rust;
```

Use `yaml::YamlLoader` to load the YAML documents and access it
as Vec/HashMap:

```rust
extern crate strict_yaml_rust;
use strict_yaml_rust::{YamlLoader, YamlEmitter};

fn main() {
    let s =
"
foo:
    - list1
    - list2
bar:
    - 1
    - 2.0
";
    let docs = YamlLoader::load_from_str(s).unwrap();

    // Multi document support, doc is a yaml::Yaml
    let doc = &docs[0];

    // Debug support
    println!("{:?}", doc);

    // Index access for map & array
    assert_eq!(doc["foo"][0].as_str().unwrap(), "list1");
    assert_eq!(doc["bar"][1].as_str().unwrap(), "2.0");

    // Chained key/array access is checked and won't panic,
    // return BadValue if they are not exist.
    assert!(doc["INVALID_KEY"][100].is_badvalue());

    // Dump the YAML object
    let mut out_str = String::new();
    {
        let mut emitter = YamlEmitter::new(&mut out_str);
        emitter.dump(doc).unwrap(); // dump the YAML object to a String
    }
    println!("{}", out_str);
}
```

Note that `yaml_rust::Yaml` implements `Index<&'a str>` & `Index<usize>`:

* `Index<usize>` assumes the container is an Array
* `Index<&'a str>` assumes the container is a string to value Map
* otherwise, `Yaml::BadValue` is returned

If your document does not conform to this convention (e.g. map with
complex type key), you can use the `Yaml::as_XXX` family API to access your
documents.

## Features

* Pure Rust
* Ruby-like Array/Hash access API
* Low-level YAML events emission

## Specification Compliance

This implementation aims to provide StrictYAML parser fully compatible with
the StrictYAML specification. 

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Fork & PR on Github.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
