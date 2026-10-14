#![allow(unreachable_patterns)]

enum State {
    Init,
    Ready,
}

fn defensive_match(state: State) -> u32 {
    match state {
        State::Init => 0,
        State::Ready => 1,
        _ => unreachable!("explicit variants already exhaustive"),
    }
}

enum Mode {
    A,
    B,
}

fn incomplete_match(mode: Mode) -> u32 {
    match mode {
        Mode::A => 1,
        _ => unreachable!("Mode::B still reaches this arm"),
    }
}

fn main() {
    let _ = defensive_match(State::Init);
    let _ = incomplete_match(Mode::A);
}
