pub const fn value() -> u8 {
    feature_helper::value()
}

#[cfg(feature = "extra")]
pub const fn optional_value() -> u8 {
    feature_optional::value()
}
