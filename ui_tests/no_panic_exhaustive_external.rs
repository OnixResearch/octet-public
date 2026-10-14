#![allow(unreachable_patterns)]

use std::cmp::Ordering;

fn external_ordering(ordering: Ordering) -> u8 {
    match ordering {
        Ordering::Less => 0,
        Ordering::Equal => 1,
        Ordering::Greater => 2,
        _ => unreachable!("external enum should still warn"),
    }
}

fn main() {
    let _ = external_ordering(Ordering::Less);
}
