#![allow(non_trait_imports, reason = "fixture targets another lint; the import and path-naming lints have their own fixtures")]

use std::path::Path;

// This file lives in `ui_tests/impure_call_in_core_sibling_scope/`, outside the
// configured `ui_tests/impure_call_in_core` scope, so the call is shell code.
fn load_shell_config(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|_error| String::new())
}

fn main() {
    let _config = load_shell_config(Path::new("config.txt"));
}
