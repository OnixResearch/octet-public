// The command line passes `-D numeric_units`; the consumer `warn` level in
// `dylint.toml` must not weaken that explicit command-line deny.
fn bare_timeout() {
    let timeout = 5000;
    let _ = timeout;
}

fn main() {
    bare_timeout();
}
