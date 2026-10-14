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
fn wildcard_transition(state: State, event: Event) -> Result<State, InvalidTransition> {
    match (state, event) {
        (State::Idle, Event::Start) => Ok(State::Running),
        _ => Err(InvalidTransition),
    }
}

#[octet::state_machine_core]
fn mutation_transition(state: State, event: Event) -> Result<State, InvalidTransition> {
    let mut next = state;
    match (state, event) {
        (State::Idle, Event::Start) => {
            next = State::Running;
            Ok(next)
        }
        (State::Idle, Event::Stop) => Err(InvalidTransition),
        (State::Running, Event::Start) => Err(InvalidTransition),
        (State::Running, Event::Stop) => {
            next = State::Idle;
            Ok(next)
        }
    }
}

#[octet::state_machine_core]
fn unsupported_transition(state: State, event: Event) -> Result<State, InvalidTransition> {
    if matches!(state, State::Idle) && matches!(event, Event::Start) {
        Ok(State::Running)
    } else {
        Err(InvalidTransition)
    }
}

fn main() {
    match wildcard_transition(State::Idle, Event::Start) {
        Ok(_) | Err(_) => {}
    }
    match mutation_transition(State::Running, Event::Stop) {
        Ok(_) | Err(_) => {}
    }
    match unsupported_transition(State::Idle, Event::Stop) {
        Ok(_) | Err(_) => {}
    }
}
