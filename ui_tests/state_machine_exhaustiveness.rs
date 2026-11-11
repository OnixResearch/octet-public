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

mod fsm_core {
    #[octet::state_machine_enum]
    #[derive(Clone, Copy)]
    pub enum Phase {
        Waiting,
        Retrying { retry_count: u8, attempt_limit: u8 },
    }

    #[octet::state_machine_enum]
    #[derive(Clone, Copy)]
    pub enum Signal {
        Tick,
        Reset(u8, u8),
    }
}

// Snake_case field bindings, underscored module paths, and `_`/`..` inside variant payloads are
// explicit arms, not wildcard fallbacks.
#[octet::state_machine_core]
fn payload_transition(phase: fsm_core::Phase, signal: fsm_core::Signal) -> Result<fsm_core::Phase, InvalidTransition> {
    match (phase, signal) {
        (fsm_core::Phase::Waiting, fsm_core::Signal::Tick) => Ok(fsm_core::Phase::Waiting),
        (fsm_core::Phase::Waiting, fsm_core::Signal::Reset(_, _)) => Err(InvalidTransition),
        (fsm_core::Phase::Retrying { retry_count, attempt_limit }, fsm_core::Signal::Tick) => {
            Ok(fsm_core::Phase::Retrying { retry_count, attempt_limit })
        }
        (fsm_core::Phase::Retrying { .. }, fsm_core::Signal::Reset(..)) => Ok(fsm_core::Phase::Waiting),
    }
}

/// An ordinary helper: prose naming `octet::state_machine_core` does not opt it into the checks.
fn prose_documented_helper(state: State, event: Event) -> State {
    match (state, event) {
        (State::Idle, Event::Start) => State::Running,
        _ => state,
    }
}

fn main() {
    match transition(State::Idle, Event::Start) {
        Ok(_) | Err(_) => {}
    }
}
