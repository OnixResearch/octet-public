use std::io::Write as _;
// Should NOT warn: `Hash` names both a trait and its derive macro; the trait
// import is what brings `.hash()` into scope.
use std::hash::Hash;

fn write_message(mut output: Vec<u8>) -> std::io::Result<Vec<u8>> {
    output.write_all(b"ok")?;
    Ok(output)
}

fn hash_message(message: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    message.hash(&mut hasher);
    std::hash::Hasher::finish(&hasher)
}

fn main() -> std::io::Result<()> {
    let _ = write_message(Vec::new())?;
    let _ = hash_message("ok");
    Ok(())
}
