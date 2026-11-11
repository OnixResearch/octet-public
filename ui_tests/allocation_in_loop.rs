#![warn(allocation_in_loop)]

const LOOP_LIMIT: usize = 4;
const SAMPLE_ITEMS: &[u8] = &[1, 2, 3];

fn allocate_in_loop(limit: usize) {
    let mut index = 0;
    while index < limit {
        // Should warn: `with_capacity` allocates on every iteration.
        let values: Vec<u8> = Vec::with_capacity(limit);
        std::hint::black_box(values);
        // Should warn: `Box::new` allocates.
        let boxed = Box::new(index);
        std::hint::black_box(boxed);
        // Should warn: `String::from` copies into a new heap buffer.
        let label = String::from("item");
        std::hint::black_box(label);
        // Should warn: `to_string` allocates a `String`.
        let rendered = index.to_string();
        std::hint::black_box(rendered);
        // Should NOT warn: empty constructors do not allocate.
        let empty: Vec<u8> = Vec::new();
        std::hint::black_box(empty);
        index += 1;
    }
}

fn copy_in_loop(items: &[u8]) {
    for _ in items {
        // Should warn: `to_vec` allocates a `Vec`.
        let copy = items.to_vec();
        std::hint::black_box(copy);
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
    let values: Vec<u8> = Vec::with_capacity(LOOP_LIMIT);
    std::hint::black_box(values);
}

struct Item {
    name: &'static str,
}

fn find_invalid(items: &[Item]) -> Result<(), String> {
    for item in items {
        if item.name.is_empty() {
            // Should NOT warn: the allocation runs once, on the exit path.
            return Err(item.name.to_string());
        }
    }
    Ok(())
}

fn first_label(items: &[Item]) -> String {
    let mut index = 0;
    loop {
        if index < items.len() {
            // Should NOT warn: `break` leaves the loop with the allocation.
            break items[index].name.to_string();
        }
        index += 1;
    }
}

fn labels_per_outer_iteration(items: &[Item]) {
    for item in items {
        // Should warn: the inner `break` only leaves the inner loop, so the
        // outer loop still allocates on every iteration.
        let label = loop {
            break item.name.to_string();
        };
        std::hint::black_box(label);
    }
}

fn labels_in_closure(items: &[Item]) {
    for item in items {
        // Should warn: `return` only leaves the closure, which runs per iteration.
        let render = || {
            return item.name.to_string();
        };
        std::hint::black_box(render());
    }
}

fn main() {
    allocate_in_loop(LOOP_LIMIT);
    copy_in_loop(SAMPLE_ITEMS);
    reuse_capacity(SAMPLE_ITEMS);
    allocate_outside_loop();
    let items = [Item { name: "" }];
    if let Err(error) = find_invalid(&items) {
        std::hint::black_box(error);
    }
    let _ = first_label(&items);
    labels_per_outer_iteration(&items);
    labels_in_closure(&items);
}
