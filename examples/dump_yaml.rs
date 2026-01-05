use std::env;
use std::fs::read_to_string;
use strict_yaml_rust::{StrictYamlEmitter, StrictYamlLoader};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().into_iter();
    args.next();

    let filename = args.next().expect("Name of file to parse");
    let s = read_to_string(filename)?;

    let docs = StrictYamlLoader::load_from_str(&s)?;

    let mut buf = String::new();

    let mut emitter = StrictYamlEmitter::new(&mut buf);

    for doc in &docs {
        emitter.dump(doc)?;
    }

    println!("{}", buf);

    Ok(())
}
