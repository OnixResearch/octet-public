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

enum Payload {
    Number(u8),
    Named { id: u8 },
    Empty,
}

fn refutable_tuple_variant_pattern(payload: Payload) -> u32 {
    match payload {
        Payload::Number(0) => 0,
        Payload::Named { .. } => 1,
        Payload::Empty => 2,
        _ => unreachable!("Payload::Number(1) still reaches this arm"),
    }
}

fn refutable_struct_variant_pattern(payload: Payload) -> u32 {
    match payload {
        Payload::Number(_) => 0,
        Payload::Named { id: 7 } => 1,
        Payload::Empty => 2,
        _ => unreachable!("Payload::Named with id 0 still reaches this arm"),
    }
}

fn irrefutable_variant_subpatterns(payload: Payload) -> u32 {
    match payload {
        Payload::Number(n) => u32::from(n),
        Payload::Named { id: _ } => 1,
        Payload::Empty => 2,
        _ => unreachable!("explicit variants with irrefutable fields already exhaustive"),
    }
}

fn main() {
    let _ = defensive_match(State::Init);
    let _ = incomplete_match(Mode::A);
    let _ = refutable_tuple_variant_pattern(Payload::Empty);
    let _ = refutable_struct_variant_pattern(Payload::Empty);
    let _ = irrefutable_variant_subpatterns(Payload::Empty);
}
