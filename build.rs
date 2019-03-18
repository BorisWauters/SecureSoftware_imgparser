// Workaround for a bug in the current stable channel
fn main() {
    println!("cargo:rustc-link-lib=shell32");
}