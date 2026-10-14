use std::io::Write as _;

fn write_message(mut output: Vec<u8>) -> std::io::Result<Vec<u8>> {
    output.write_all(b"ok")?;
    Ok(output)
}

fn main() -> std::io::Result<()> {
    let _ = write_message(Vec::new())?;
    Ok(())
}
