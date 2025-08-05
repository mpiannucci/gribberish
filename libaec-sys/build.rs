use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // Always build libaec from source for predictability and simplicity
    build_libaec_from_source();

    // Generate bindings
    let out_dir = env::var("OUT_DIR").unwrap();
    let include_path = format!("{}/include", out_dir);

    let bindings = bindgen::Builder::default()
        .header(format!("{include_path}/libaec.h"))
        .clang_arg(format!("-I{}", include_path))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(out_dir);
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn build_libaec_from_source() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let libaec_dir = format!("{}/libaec", out_dir);

    // Check if libaec source already exists
    if !std::path::Path::new(&libaec_dir).exists() {
        // Download libaec source
        let status = std::process::Command::new("git")
            .args(&[
                "clone",
                "https://gitlab.dkrz.de/k202009/libaec.git",
                &libaec_dir,
                "--depth=1",
                "--branch=v1.1.4",
            ])
            .status()
            .expect("Failed to clone libaec repository. Please ensure git is installed.");

        if !status.success() {
            panic!("Failed to clone libaec repository");
        }
    }

    // Build libaec using cmake
    let dst = cmake::build(libaec_dir);
    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=aec");
}
