strict-yaml-rust
-----
A [StrictYAML](http://hitchdev.com/strictyaml/) implementation for Rust 
obtained by savagely chopping off code from [yaml-rust](https://crates.io/crates/yaml-rust).

Now with built-in `serde` support as an optional feature! (@puetzp)

[![license](https://img.shields.io/crates/l/strict-yaml-rust.svg)](https://crates.io/crates/strict-yaml-rust/)
[![version](https://img.shields.io/crates/v/strict-yaml-rust.svg)](https://crates.io/crates/strict-yaml-rust/)

## What is StrictYAML?

StrictYAML is a subset of the YAML format, removing troublesome parts of the specification:

 - No typing. StrictYAML only knows Strings, Arrays and Dicts (Maps)  
 - No duplicate keys
 - No tags
 - No anchors or refs
 - No "flow" style (embedded JSON)
 
In short, keeping only the parts of YAML that we all know and love.

For more details, see the original documentation and implementation:

 - http://hitchdev.com/strictyaml/
 - https://github.com/crdoconnor/strictyaml
 
## Caveats

Missing from this implementation is a 
[programmatic schema](https://hitchdev.com/strictyaml/using/alpha/howto/either-or-validation/), 
which would enable document validation and typed value parsing. 
Seems that you'll still have to parse integers and dates from strings, 
or [send a PR](https://github.com/fralalonde/strict-yaml-rust/pulls)!

## Quick Start

Add the following to the Cargo.toml of your project:

```toml
[dependencies]
strict-yaml-rust = "0.2"
```

or

```toml
[dependencies.yaml-rust]
git = "https://github.com/fralalonde/strict-yaml-rust.git"
```

Use `StrictYamlLoader` to load the YAML documents, access it as Vec/HashMap and finally print it to `stdout` via `StrictYamlEmitter`:

```rust
use strict_yaml_rust::{StrictYamlEmitter, StrictYamlLoader};

fn main() {
    let s = "
foo:
    - list1
    - list2
bar:
    - 1
    - 2.0
";
    let docs = StrictYamlLoader::load_from_str(s).unwrap();

    // Multi document support, doc is a yaml::Yaml
    let doc = &docs[0];

    // Debug support
    println!("{:?}", doc);

    // Index access for map & array
    assert_eq!(doc["foo"][0].as_str().unwrap(), "list1");
    assert_eq!(doc["bar"][1].as_str().unwrap(), "2.0");

    // Chained key/array access is checked and won't panic,
    // return BadValue if a key does not exist.
    assert!(doc["INVALID_KEY"][100].is_badvalue());

    // Dump the YAML object
    let mut out_str = String::new();

    {
        let mut emitter = StrictYamlEmitter::new(&mut out_str);
        emitter.dump(doc).unwrap(); // dump the YAML object to a String
    }

    println!("{}", out_str);
}
```

Note that `strict_yaml_rust::StrictYaml` implements `Index<&'a str>` AND `Index<usize>`:

* `Index<usize>` assumes the container is an Array
* `Index<&'a str>` assumes the container is a string to value Map
* otherwise, `Yaml::BadValue` is returned

## Features

* Pure Rust
* Ruby-like Array/Hash access API
* Low-level StrictYAML events emission

## Specification Compliance

This implementation aims to provide StrictYAML parser fully compatible with the StrictYAML specification. 

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
