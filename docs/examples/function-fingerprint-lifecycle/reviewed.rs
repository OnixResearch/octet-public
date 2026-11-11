// octet-fingerprint: b3:90089d987ce66794a19701cdb27dfecee5a5283ed5d3bd3dd2de2b102fd7edb9; review=safety-critical; evidence=docs/examples/function-fingerprint-lifecycle/lifecycle-transcript.md; macro=allow
#[rustfmt::skip]
pub fn bounded_retry_delay_ms(attempt: u32) -> u32 {
    assert!(attempt <= 5);
    75_u32.saturating_mul(attempt.saturating_add(1))
}
