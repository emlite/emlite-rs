fn main() {
    println!("cargo:rerun-if-changed=emsenv/lib.c");
    println!("cargo:rerun-if-changed=emsenv/CMakeLists.txt");
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os == "emscripten" {
        const TOOLCHAIN_SUBPATH: &str = "cmake/Modules/Platform/Emscripten.cmake";
        let mut emscripten_root =
            std::path::PathBuf::from(std::env::var("EMSCRIPTEN_ROOT").unwrap_or_default());
        if !emscripten_root.exists() {
            emscripten_root = std::path::PathBuf::from(
                std::env::var("EMSDK").expect("Neither EMSDK nor EMSCRIPTEN_ROOT were set."),
            )
            .join("upstream/emscripten");
        }
        let toolchain_file = emscripten_root.join(TOOLCHAIN_SUBPATH);
        let dst = cmk::Config::new("emsenv")
            .define("CMAKE_TOOLCHAIN_FILE", toolchain_file)
            .profile("Release")
            .build();
        println!("cargo:rustc-link-search=naive={}", dst.display());
        println!("cargo:rustc-link-lib=static=emsenv");
    }
}
