//! The violating branch is selected when the strict feature is disabled.

#[cfg(not(feature = "strict"))]
pub fn select() -> u32 {
    infra_api::Database::connect().query()
}

#[cfg(feature = "strict")]
pub fn select() -> u32 {
    0
}
