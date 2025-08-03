use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wrapper.h");

    // Always build libaec from source for predictability and simplicity
    build_libaec_from_source();

    // Generate bindings
    let out_dir = env::var("OUT_DIR").unwrap();
    let include_path = format!("{}/include", out_dir);

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
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
                "--branch=v1.1.4", // Use specific stable version
            ])
            .status()
            .expect("Failed to clone libaec repository. Please ensure git is installed.");

        if !status.success() {
            panic!("Failed to clone libaec repository");
        }
    }

    // Build libaec using cmake
    let dst = cmake::Config::new(&libaec_dir)
        .define("BUILD_SHARED_LIBS", "OFF") // Static linking
        .define("CMAKE_BUILD_TYPE", "Release") // Optimized build
        .define("CMAKE_POSITION_INDEPENDENT_CODE", "ON") // Needed for static linking in shared libs
        .build();

    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=aec");
    println!("cargo:include={}/include", dst.display());
}
