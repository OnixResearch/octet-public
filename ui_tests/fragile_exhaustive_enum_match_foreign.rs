#![warn(fragile_exhaustive_enum_match)]

fn classify(ordering: std::cmp::Ordering) -> bool {
    match ordering {
        std::cmp::Ordering::Less => true,
        std::cmp::Ordering::Equal => true,
        std::cmp::Ordering::Greater => false,
    }
}

fn main() {}
