extern crate strict_yaml_rust;
#[macro_use]
extern crate quickcheck;

use quickcheck::TestResult;
use strict_yaml_rust::{StrictYaml, StrictYamlEmitter, StrictYamlLoader};

quickcheck! {
    fn test_check_weird_keys(xs: Vec<String>) -> TestResult {
        let mut out_str = String::new();
        {
            let mut emitter = StrictYamlEmitter::new(&mut out_str);
            emitter.dump(&StrictYaml::Array(xs.into_iter().map(StrictYaml::String).collect())).unwrap();
        }
        if let Err(err) = StrictYamlLoader::load_from_str(&out_str) {
            return TestResult::error(format!("{}", err));
        }
        TestResult::passed()
    }
}
