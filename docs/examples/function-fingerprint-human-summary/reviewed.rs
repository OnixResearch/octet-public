// octet-fingerprint: b3:cef46dc019ce1c2c90751fba2d6d283bcd1c3763b7edd3c8d99d1e714c5ed71a; review=operator-human-summary; evidence=docs/examples/function-fingerprint-human-summary/transcript.md
pub fn capped_batch_size(requested: u32) -> u32 {
    requested.min(256)
}

// octet-fingerprint: b3:7b3c4c3232600191b80357e69b7c37a176123b4c223d1d5e15c8ad684de5aa95; review=operator-human-summary; evidence=docs/examples/function-fingerprint-human-summary/transcript.md; macro=allow
pub fn describe_batch(size: u32) -> String {
    format!("batch={size}")
}
