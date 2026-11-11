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

fn ignore_result_underscore_assignment() {
    let mut buf = Vec::new();
    // Should warn: `_ = ...` discards the Result like `let _ = ...`.
    _ = buf.write_all(b"hello");
}

struct Listener;

impl Listener {
    // Should NOT warn: a `_` async fn parameter is not a discarded value, even
    // though rustc lowers it into `let _ = ...` inside the async body.
    async fn on_done(&self, _: std::io::Result<()>) {}
}

fn main() {
    ignore_result();
    ignore_result_statement_position();
    ignore_non_result();
}
