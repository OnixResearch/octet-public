// Module loaded by `unjustified_allow.rs` from a directory whose path contains
// "clippy". The lint scope comes from the attribute's lint paths, never from
// the source file path, so this rustc lint suppression stays quiet.

#[allow(dead_code)]
pub fn rustc_lint_allow_under_clippy_path() {}
