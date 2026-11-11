//! The compliant branch is selected when the strict feature is enabled.

#[cfg(feature = "strict")]
pub fn select(port: &dyn core_api::Port) -> u32 {
    port.run()
}

#[cfg(not(feature = "strict"))]
pub fn select(_port: &dyn core_api::Port) -> u32 {
    0
}
