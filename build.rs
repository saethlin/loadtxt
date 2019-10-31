fn main() {
    println!("cargo:rerun-if-changed=src/lib.rs");
    std::process::Command::new("cbindgen")
        .arg("-lC")
        .arg("-otarget/loadtxt.h")
        .output()
        .unwrap();
}
