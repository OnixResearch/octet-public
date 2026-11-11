pub fn identity(value: u32) -> u32 {
    value
}

#[cfg(test)]
macro_rules! hidden_read {
    () => {
        std::fs::read("never-executed")
    };
}

#[cfg(test)]
#[test]
fn test_only_effect() {
    let _ = hidden_read!();
}

#[cfg(feature = "extra")]
pub fn feature_only_effect() {
    let _ = std::fs::read("never-executed");
}
