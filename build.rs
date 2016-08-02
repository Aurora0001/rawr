extern crate serde_codegen;

use std::path::Path;
use std::fs;

pub fn main() {
    for path in fs::read_dir("src/responses/").unwrap() {
        let path = path.unwrap().path();
        if path.extension().unwrap() == "in" && path.file_name().unwrap() != "mod.rs.in" {
            let dest = Path::new("src/responses").join(path.file_stem().unwrap()).with_extension("rs.out");
            serde_codegen::expand(path, dest).unwrap();
        }
    }

    serde_codegen::expand("src/responses/mod.rs.in", "src/responses/mod.rs.out").unwrap();
}
