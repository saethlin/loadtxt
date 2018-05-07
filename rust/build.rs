extern crate cbindgen;
use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut config = cbindgen::Config::default();
    config.language = cbindgen::Language::C;
    if let Ok(bindings) = cbindgen::generate_with_config(&crate_dir, config) {
        bindings.write_to_file("target/loadtxt.h");
    }
}
