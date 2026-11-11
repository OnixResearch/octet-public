fn explicit_break_loop() {
    // Should NOT warn: structural exit is visible.
    loop {
        if true {
            break;
        }
    }
}

fn marker_only_runaway_loop() {
    // Should warn: marker names alone are not a bound.
    let mut attempt = 0;
    loop {
        attempt += 1;
        std::hint::black_box(attempt);
    }
}

fn counter_loop() {
    // Should NOT warn: the counter guards a structural `break`.
    let mut attempt = 0;
    loop {
        attempt += 1;
        if attempt > 3 {
            break;
        }
    }
}

fn max_retries_loop() {
    // Should NOT warn: the MAX_RETRIES bound guards a structural `break`.
    const MAX_RETRIES: u32 = 3;
    let mut i = 0;
    loop {
        i += 1;
        if i >= MAX_RETRIES {
            break;
        }
    }
}

fn main() {
    explicit_break_loop();
    marker_only_runaway_loop();
    counter_loop();
    max_retries_loop();
}

fn bounded_for_loop() {
    // Should NOT warn: `for` has an explicit bound
    for _ in 0..3 {
        std::hint::spin_loop();
    }
}

fn inner_for_return_loop(items: &[u32]) -> u32 {
    // Should NOT warn: `return` inside the inner `for` exits the outer loop.
    loop {
        for item in items {
            if *item > 3 {
                return *item;
            }
        }
        std::hint::black_box(items);
    }
}

fn labelled_outer_break_loop(items: &[u32]) {
    // Should NOT warn: `break 'outer` from the inner loop exits the outer loop.
    'outer: loop {
        loop {
            if std::hint::black_box(items.is_empty()) {
                break 'outer;
            }
            std::hint::black_box(items);
        }
    }
}

fn inner_break_only_loop(items: &[u32]) {
    // Should warn: unlabelled `break` only exits the inner `for`.
    loop {
        for item in items {
            if *item > 3 {
                break;
            }
        }
        std::hint::black_box(items);
    }
}

fn inner_labelled_break_only_loop(items: &[u32]) {
    // Should warn: `break 'inner` only exits the inner loop.
    loop {
        'inner: loop {
            if std::hint::black_box(items.is_empty()) {
                break 'inner;
            }
        }
        std::hint::black_box(items);
    }
}

fn labelled_block_break_only_loop(items: &[u32]) {
    // Should warn: `break 'block` only exits the labelled block.
    loop {
        'block: {
            if std::hint::black_box(items.is_empty()) {
                break 'block;
            }
            std::hint::black_box(items);
        }
    }
}

fn next_reading(raw: u32) -> Result<u32, String> {
    if raw > 3 { Err(format!("reading {raw} out of range")) } else { Ok(raw) }
}

fn try_operator_exit_loop(raw: u32) -> Result<(), String> {
    // Should NOT warn: `?` lowers to a reachable `return` on the error path.
    loop {
        let reading = next_reading(std::hint::black_box(raw))?;
        std::hint::black_box(reading);
    }
}

macro_rules! ensure {
    ($cond:expr, $err:expr) => {
        if !$cond {
            return Err($err);
        }
    };
}

fn ensure_macro_exit_loop(raw: u32) -> Result<(), String> {
    // Should NOT warn: the `ensure!` expansion returns when the check fails.
    loop {
        ensure!(std::hint::black_box(raw) < 3, "raw reading out of range".to_owned());
    }
}
