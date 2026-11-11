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

impl LocalState {
    fn is_ready(&self) -> bool {
        match self {
            LocalState::Ready => true,
            LocalState::Closed => false,
        }
    }
}

/// Prose that names `octet::sealed_enum` does not seal the enum.
enum DocumentedState {
    Open,
    Shut,
}

fn documented_is_open(state: DocumentedState) -> bool {
    match state {
        DocumentedState::Open => true,
        DocumentedState::Shut => false,
    }
}

fn main() {}
