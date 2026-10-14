fn main() {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR");
    let path = std::path::PathBuf::from(out_dir).join("extra_symbols.rs");
    std::fs::write(path, "const EXTRA_SYMBOLS: &[&str] = &[];").expect("write extra_symbols.rs");
}
