// octet-fingerprint: b3:c2cb392a462f04b9e56590bc9f2941b9724e16647901322fa15c00928af7d687; review=operator-human-summary; evidence=docs/examples/function-fingerprint-human-summary/transcript.md
pub fn capped_batch_size(requested: u32) -> u32 {
    requested.min(256)
}

// octet-fingerprint: b3:e41c5b7b5d94059a506351f35048fa77df8b9c53c3955c8890cbb2212e037876; review=operator-human-summary; evidence=docs/examples/function-fingerprint-human-summary/transcript.md; macro=allow
pub fn describe_batch(size: u32) -> String {
    format!("batch={size}")
}
