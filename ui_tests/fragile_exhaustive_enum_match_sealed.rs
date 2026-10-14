#![feature(register_tool)]
#![register_tool(octet)]
#![warn(fragile_exhaustive_enum_match)]

#[octet::sealed_enum]
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
