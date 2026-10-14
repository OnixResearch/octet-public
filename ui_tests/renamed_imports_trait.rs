use std::fmt::Write as _;

fn write_message(mut output: String) -> std::fmt::Result {
    output.write_str("ok")?;
    Ok(())
}

fn main() -> std::fmt::Result {
    write_message(String::new())
}
