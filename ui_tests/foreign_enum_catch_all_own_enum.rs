#![warn(foreign_enum_catch_all)]

enum LocalState {
    Ready,
    Closed,
}

fn classify(state: LocalState) -> bool {
    match state {
        LocalState::Ready => true,
        _ => false,
    }
}

fn main() {}
