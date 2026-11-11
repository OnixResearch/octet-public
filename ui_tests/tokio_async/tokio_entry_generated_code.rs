#![feature(register_tool)]
#![register_tool(tigerstyle)]
#![warn(
    tigerstyle::safety,
    tigerstyle::naming,
    tigerstyle::performance,
    tigerstyle::structure,
    tigerstyle::purity,
    tigerstyle::concurrency
)]
// Tokio's generated `#[allow(clippy::expect_used, ..)]` is covered by the
// `unjustified_allow` fixtures; this fixture tracks the generated runtime code.
#![allow(tigerstyle::unjustified_allow)]

// `#[tokio::main]` and `#[tokio::test]` build their runtime with
// `quote_spanned!` at the last statement of the user's body, so the generated
// `.build().expect("Failed building the Runtime")` carries a user span. Only the
// `.unwrap()` calls written below are findings.

fn read_name(raw: &str) -> Option<String> {
    Some(raw.to_owned())
}

macro_rules! name_or_panic {
    ($raw:expr) => {
        read_name($raw).unwrap()
    };
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let name = read_name("octet");
    drop(name);
}

#[tokio::main(flavor = "current_thread")]
async fn serve_ending_in_branch() {
    let name = read_name("octet");
    if name.is_some() {
        drop(name);
    }
}

#[tokio::main(flavor = "current_thread")]
async fn serve_returning_result() -> Result<(), String> {
    let name = read_name("octet").ok_or_else(|| String::from("missing name"))?;
    drop(name);
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn serve_with_user_unwrap() {
    // Should error: the user wrote this `.unwrap()`.
    let name = read_name("octet").unwrap();
    drop(name);
}

#[tokio::main(flavor = "current_thread")]
async fn serve_with_macro_unwrap() {
    // Should error: the user's `macro_rules!` body wrote this `.unwrap()`.
    let name = name_or_panic!("octet");
    drop(name);
}

#[tokio::test]
async fn runtime_drives_test_body() {
    let name = read_name("octet");
    drop(name);
}
