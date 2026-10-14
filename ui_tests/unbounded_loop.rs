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
    // Should NOT warn: contains "attempt"
    let mut attempt = 0;
    loop {
        attempt += 1;
        if attempt > 3 {
            break;
        }
    }
}

fn max_retries_loop() {
    // Should NOT warn: contains MAX_RETRIES
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
