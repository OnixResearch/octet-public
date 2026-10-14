#![allow(unreachable_patterns)]

enum State {
    Init,
    Ready,
}

fn indirect_unreachable(state: State) -> u8 {
    match state {
        State::Init => 0,
        State::Ready => 1,
        _ => {
            let _note = "still should warn";
            unreachable!("indirect wildcard body should still warn");
        }
    }
}

fn main() {
    let _ = indirect_unreachable(State::Init);
}
