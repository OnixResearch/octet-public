#![deny(no_unwrap)]

fn uses_unwrap() {
    let val: Result<u32, &str> = Ok(1);
    let _ = val.unwrap();
}

fn uses_expect() {
    let val: Option<u32> = Some(1);
    let _ = val.expect("missing");
}

fn main() {
    uses_unwrap();
    uses_expect();
}
