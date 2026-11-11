// octet-fingerprint: b3:ea3ca43821de8c24558ea6c5d43beae6c6731f49e62ebc6b2675f96e4c2019f7; review=json-script-example; evidence=docs/examples/function-fingerprint-json-output/transcript.md; macro=allow
pub fn capped_retries(requested: u32) -> u32 {
    requested.min(5)
}
