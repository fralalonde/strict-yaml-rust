extern crate strict_yaml_rust;

use std::env;
use std::fs::{File, read_to_string};

use std::io;
use strict_yaml_rust::yaml;

pub type Result<T> = ::std::result::Result<T, Box<::std::error::Error>>;

fn print_indent(indent: usize) {
    for _ in 0..indent {
        print!("    ");
    }
}

fn dump_node(doc: &yaml::StrictYaml, indent: usize) {
    match *doc {
        yaml::StrictYaml::Array(ref v) => {
            for x in v {
                dump_node(x, indent + 1);
            }
        },
        yaml::StrictYaml::Hash(ref h) => {
            for (k, v) in h {
                print_indent(indent);
                println!("{:?}:", k);
                dump_node(v, indent + 1);
            }
        },
        _ => {
            print_indent(indent);
            println!("{:?}", doc);
        }
    }
}

fn main() -> Result<()> {
    let mut args = env::args().into_iter();
    args.next();

    let filename = args.next().expect("Name of file to parse");
    let s = read_to_string(filename)?;

    let docs = yaml::StrictYamlLoader::load_from_str(&s)?;
    for doc in &docs {
        println!("---");
        dump_node(doc, 0);
    }
    Ok(())
}
