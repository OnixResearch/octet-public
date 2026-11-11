#![allow(unreachable_patterns)]
#![allow(non_trait_imports, reason = "fixture targets another lint; the import and path-naming lints have their own fixtures")]

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
