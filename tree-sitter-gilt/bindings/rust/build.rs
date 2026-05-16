fn main() {
    let mut config = cc::Build::new();
    config.compiler("clang");
    config.include("src");
    config.file("src/parser.c");

    if std::env::var("TARGET").unwrap_or_default() == "wasm32-wasip2" {
        if let Ok(wasi_sdk_path) = std::env::var("WASI_SDK_PATH") {
            let sysroot = format!("{}/share/wasi-sysroot", wasi_sdk_path);
            config.flag(&format!("--sysroot={}", sysroot));
        }
    }

    if let Ok(file) = std::fs::File::open("src/scanner.c") {
        drop(file);
        config.file("src/scanner.c");
    }

    config.compile("tree-sitter-gilt");
}
