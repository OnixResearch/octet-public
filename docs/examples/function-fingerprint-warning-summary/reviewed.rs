// octet-fingerprint: b3:4bb0a5a53fdcdada0290b6d4f90cca72ad0635e0b535fe2ca09374d8986ab32f; review=macro-warning-example; evidence=docs/examples/function-fingerprint-warning-summary/transcript.md; macro=review-required
pub fn unreviewed_macro_path(size: u32) -> String {
    format!("size={size}")
}

// octet-fingerprint: b3:83b2e51ca9854736155771a41c9e116ca71043f78c367f84fe1e4773d299f816; review=TODO; evidence=TODO
pub fn missing_gate_metadata(limit: u32) -> u32 {
    limit.min(64)
}
