// octet-fingerprint: b3:9edc0f2555bf5f15be3d7b09806228695671427c8f8720f466af4280b4bd1130; review=json-script-example; evidence=docs/examples/function-fingerprint-json-output/transcript.md; macro=allow
pub fn capped_retries(requested: u32) -> u32 {
    requested.min(5)
}
