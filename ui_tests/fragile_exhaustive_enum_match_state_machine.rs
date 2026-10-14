#![feature(register_tool)]
#![register_tool(octet)]
#![warn(fragile_exhaustive_enum_match)]

#[octet::state_machine_enum]
enum State {
    Idle,
    Running,
}

fn terminal(state: State) -> bool {
    match state {
        State::Idle => false,
        State::Running => true,
    }
}

fn main() {
    let _terminal = terminal(State::Idle);
}
