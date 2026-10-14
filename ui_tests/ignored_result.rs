use std::io::Write;

fn ignore_result() {
    let mut buf = Vec::new();
    // Should warn: Result discarded with let _ = ...
    let _ = buf.write_all(b"hello");
}

fn ignore_result_statement_position() {
    let mut buf = Vec::new();
    // Should warn: Result discarded as a statement.
    buf.write_all(b"hello");
}

fn ignore_non_result() {
    // Should NOT warn: not a Result type
    let _ = 42;
}

fn handle_result() -> std::io::Result<()> {
    let mut buf = Vec::new();
    // Should NOT warn: Result is propagated
    buf.write_all(b"hello")?;
    Ok(())
}

fn main() {
    ignore_result();
    ignore_result_statement_position();
    ignore_non_result();
}
