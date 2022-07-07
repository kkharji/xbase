use std::env;
use std::path::PathBuf;

fn main() {
    let crate_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let config_file_path = crate_dir.join("cbindgen.toml");
    let config = cbindgen::Config::from_file(config_file_path).unwrap();
    cbindgen::generate_with_config(&crate_dir, config)
        .unwrap()
        .write_to_file("../build/libxbase.h");
}
