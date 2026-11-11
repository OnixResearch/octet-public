// A selected strict profile denies `no_unwrap`. `--force-warn no_unwrap` and
// `--cap-lints warn` would each keep it a warning, so the run fails closed.
fn first(values: &[u32]) -> u32 {
    *values.first().unwrap()
}

fn main() {
    let _ = first(&[1]);
}
