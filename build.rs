extern crate serde_codegen;

use std::env;
use std::path::Path;
use std::fs;

pub fn main() {
    for path in fs::read_dir("src/responses/").unwrap() {
        let path = path.unwrap().path();
        if path.extension().unwrap() == "in" {
            let dest = Path::new("src/responses").join(path.file_stem().unwrap());
            serde_codegen::expand(path, dest).unwrap();
        }
    }
}
