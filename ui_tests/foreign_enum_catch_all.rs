#![warn(foreign_enum_catch_all)]

fn classify(ordering: std::cmp::Ordering) -> bool {
    match ordering {
        std::cmp::Ordering::Less => true,
        _ => false,
    }
}

fn main() {}
