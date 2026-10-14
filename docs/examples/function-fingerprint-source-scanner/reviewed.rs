macro_rules! define_generated_boundary {
    ($name:ident) => {
        pub fn $name(input: u32) -> u32 {
            input.saturating_add(1)
        }
    };
}

define_generated_boundary!(generated_boundary);

// octet-fingerprint: b3:eba4de659452eb0c6179a962b1452a986b4fd6af24afb52a54badf4a250e4a42; review=source-scanner-boundary; evidence=docs/examples/function-fingerprint-source-scanner/transcript.md
pub fn explicit_generated_boundary(input: u32) -> u32 {
    generated_boundary(input).min(64)
}
