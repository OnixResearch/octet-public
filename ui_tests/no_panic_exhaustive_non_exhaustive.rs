#![allow(unreachable_patterns)]

#[non_exhaustive]
enum LocalState {
    Init,
    Ready,
}

fn local_non_exhaustive(state: LocalState) -> u8 {
    match state {
        LocalState::Init => 0,
        LocalState::Ready => 1,
        _ => unreachable!("non-exhaustive enum should still warn"),
    }
}

fn main() {
    let _ = local_non_exhaustive(LocalState::Init);
}
