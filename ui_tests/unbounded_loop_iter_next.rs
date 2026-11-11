fn iter_next_loop_still_warns(items: Vec<u32>) {
    let mut iter = items.into_iter();
    while let Some(value) = iter.next() {
        std::hint::black_box(value);
    }
}

fn check_value(value: u32) -> Result<u32, String> {
    if value > 3 { Err(format!("value {value} out of range")) } else { Ok(value) }
}

fn iter_next_loop_with_try_exit(items: Vec<u32>) -> Result<(), String> {
    // Should NOT warn: `?` lowers to a reachable `return`.
    let mut iter = items.into_iter();
    while let Some(value) = iter.next() {
        std::hint::black_box(check_value(value)?);
    }
    Ok(())
}

fn main() {
    iter_next_loop_still_warns(vec![1, 2, 3]);
    if let Err(error) = iter_next_loop_with_try_exit(vec![1, 2, 3]) {
        std::hint::black_box(error);
    }
}
