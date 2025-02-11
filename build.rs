#[cfg(not(feature = "bindings"))]
fn main() {}

#[cfg(feature = "bindings")]
fn main() {
    use std::env;
    extern crate cbindgen;

    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_language(cbindgen::Language::C)
        .generate()
        .map_or_else(
            |error| match error {
                cbindgen::Error::ParseSyntaxError { .. } => {}
                e => panic!("{:?}", e),
            },
            |bindings| {
                bindings.write_to_file("target/include/json.h");
            },
        );
}
