#![feature(register_tool)]
#![register_tool(octet)]
#![warn(state_machine_exhaustiveness)]

#[octet::state_machine_enum]
#[derive(Clone, Copy)]
enum State {
    Idle,
    Running,
}

#[octet::state_machine_enum]
#[derive(Clone, Copy)]
enum Event {
    Start,
    Stop,
}

struct InvalidTransition;

#[octet::state_machine_core]
fn transition(state: State, event: Event) -> Result<State, InvalidTransition> {
    match (state, event) {
        (State::Idle, Event::Start) => Ok(State::Running),
        (State::Idle, Event::Stop) => Err(InvalidTransition),
        (State::Running, Event::Start) => Err(InvalidTransition),
        (State::Running, Event::Stop) => Ok(State::Idle),
    }
}

fn main() {
    match transition(State::Idle, Event::Start) {
        Ok(_) | Err(_) => {}
    }
}
