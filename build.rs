use std::env;

fn main() {
    let path = env::var("Z3_LIB_DIR").unwrap();
    println!("cargo:rustc-link-search=native={}", path);
}
