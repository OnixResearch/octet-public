#[rustfmt::skip]
// octet-fingerprint: b3:7deb37887bd38ad0b5e586a9700eb9d34f61153df85ab06649c3f468f6816114; review=safety-critical; evidence=docs/examples/function-fingerprint-lifecycle/lifecycle-transcript.md; macro=allow
pub fn bounded_retry_delay_ms(attempt: u32) -> u32 {
    assert!(attempt <= 5);
    75_u32.saturating_mul(attempt.saturating_add(1))
}
