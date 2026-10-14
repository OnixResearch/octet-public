#![warn(allocation_in_loop)]

const LOOP_LIMIT: usize = 4;
const SAMPLE_ITEMS: &[u8] = &[1, 2, 3];

fn allocate_in_loop(limit: usize) {
    let mut index = 0;
    while index < limit {
        // Should warn: Vec::new allocates inside the loop body.
        let values: Vec<u8> = Vec::new();
        std::hint::black_box(values);
        index += 1;
    }
}

fn reuse_capacity(items: &[u8]) {
    // Should NOT warn: capacity is reserved before iteration and reused.
    let mut values = Vec::with_capacity(items.len());
    for item in items {
        values.clear();
        values.push(*item);
        std::hint::black_box(&values);
    }
}

fn allocate_outside_loop() {
    // Should NOT warn: the allocation is outside a loop.
    let values: Vec<u8> = Vec::new();
    std::hint::black_box(values);
}

fn main() {
    allocate_in_loop(LOOP_LIMIT);
    reuse_capacity(SAMPLE_ITEMS);
    allocate_outside_loop();
}
