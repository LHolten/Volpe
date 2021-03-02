use std::env;

fn main() {
    let path = env::var("Z3_LIB_DIR").expect("Missing Z3_LIB_DIR environment variable");
    println!("cargo:rustc-link-search=native={}", path);
}
