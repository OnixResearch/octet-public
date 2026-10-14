#![feature(register_tool)]
#![register_tool(tigerstyle)]

use std::collections::BTreeMap;
use std::collections::HashMap;

const SAMPLE_VALUE: u64 = 1;

trait RandomPort {
    fn next(&mut self) -> u64;
}

struct CounterPort;

impl RandomPort for CounterPort {
    fn next(&mut self) -> u64 {
        SAMPLE_VALUE
    }
}

struct ExplicitState {
    value: u64,
}

struct CommandSpec {
    program: String,
}

#[tigerstyle::pure]
fn clean_marked_pure(left: u64, right: u64) -> u64 {
    left + right
}

fn use_explicit_environment(home_dir: &str) -> String {
    home_dir.to_owned()
}

fn use_injected_randomness(random: &mut impl RandomPort) -> u64 {
    random.next()
}

fn return_ordered_names(names: BTreeMap<String, u64>) -> Vec<String> {
    names.keys().cloned().collect::<Vec<_>>()
}

fn sort_hash_names(names: HashMap<String, u64>) -> Vec<String> {
    let mut output = names.keys().cloned().collect::<Vec<_>>();
    output.sort();
    output
}

fn explicit_state_transition(state: ExplicitState) -> ExplicitState {
    ExplicitState { value: state.value + SAMPLE_VALUE }
}

fn describe_command(program: String) -> CommandSpec {
    CommandSpec { program }
}

fn main() {
    let mut random = CounterPort;
    let _home = use_explicit_environment("/tmp/octet");
    let _random_value = use_injected_randomness(&mut random);
    let _pure_value = clean_marked_pure(SAMPLE_VALUE, SAMPLE_VALUE);
    let _state = explicit_state_transition(ExplicitState { value: SAMPLE_VALUE });
    let spec = describe_command("echo".to_owned());
    let _program = spec.program;
}
