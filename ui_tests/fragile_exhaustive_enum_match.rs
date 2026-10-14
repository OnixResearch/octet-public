#![warn(fragile_exhaustive_enum_match)]

enum LocalState {
    Ready,
    Closed,
}

fn classify(state: LocalState) -> bool {
    match state {
        LocalState::Ready => true,
        LocalState::Closed => false,
    }
}

fn main() {}
