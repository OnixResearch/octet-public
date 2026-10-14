const SAMPLE_VALUE: u64 = 1;

fn compute_with_console_output(value: u64) -> u64 {
    println!("processing {value}");
    dbg!(value)
}

fn main() {
    let _value = compute_with_console_output(SAMPLE_VALUE);
}
