#![feature(register_tool)]
#![register_tool(tigerstyle)]
#![warn(unjustified_allow)]

// Should NOT warn: `#[tokio::main]` wraps the body in a generated
// `#[allow(clippy::...)]` block spanned at the last statement. The user never
// wrote that attribute, so it is not their suppression to justify.
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let _value = helper();
}

// Should warn: a suppression the user wrote is still governed.

#[allow(clippy::needless_return)]
fn helper() -> u32 {
    return 1;
}
