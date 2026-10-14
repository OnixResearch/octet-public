// octet-fingerprint: b3:f00700706a371a7cf92aa0ce17350dfc5d349e561a1f491d1102d5f17688b35e; review=macro-warning-example; evidence=docs/examples/function-fingerprint-warning-summary/transcript.md; macro=review-required
pub fn unreviewed_macro_path(size: u32) -> String {
    format!("size={size}")
}

// octet-fingerprint: b3:70e5f9ca896cf2aa8f305b1ea6cdd35ce59ead7f7c8cfc5fe14ff9c7db7d76f4; review=TODO; evidence=TODO
pub fn missing_gate_metadata(limit: u32) -> u32 {
    limit.min(64)
}
