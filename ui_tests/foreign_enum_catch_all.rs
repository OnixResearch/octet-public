#![warn(foreign_enum_catch_all)]

fn classify(ordering: std::cmp::Ordering) -> bool {
    match ordering {
        std::cmp::Ordering::Less => true,
        _ => false,
    }
}

fn classify_ref(ordering: &std::cmp::Ordering) -> bool {
    match ordering {
        std::cmp::Ordering::Less => true,
        _ => false,
    }
}

trait IsGreater {
    fn is_greater_than(&self) -> bool;
}

impl IsGreater for std::cmp::Ordering {
    fn is_greater_than(&self) -> bool {
        match self {
            std::cmp::Ordering::Greater => true,
            _ => false,
        }
    }
}

fn main() {}
