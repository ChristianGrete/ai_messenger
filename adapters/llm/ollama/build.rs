fn main() {
    // Set up cargo rerun triggers for WIT files
    println!("cargo:rerun-if-changed=wit/");
    println!("cargo:rerun-if-changed=../../../wit/");
}
