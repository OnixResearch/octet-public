#![warn(foreign_enum_catch_all)]

macro_rules! generated_match {
    ($ordering:expr) => {
        match $ordering {
            std::cmp::Ordering::Less => true,
            _ => false,
        }
    };
}

fn classify(ordering: std::cmp::Ordering) -> bool {
    generated_match!(ordering)
}

fn main() {}
