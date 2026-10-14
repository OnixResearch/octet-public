macro_rules! generated_identity {
    ($value:expr) => {
        $value
    };
}

pub fn macro_generated(value: u32) -> u32 {
    generated_identity!(value)
}
