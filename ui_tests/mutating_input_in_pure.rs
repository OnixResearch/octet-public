#![feature(register_tool)]
#![register_tool(tigerstyle)]

const INITIAL_COUNTER: u64 = 0;
const COUNTER_INCREMENT: u64 = 1;

#[tigerstyle::pure]
fn update_counter(counter: &mut u64) -> u64 {
    *counter += COUNTER_INCREMENT;
    *counter
}

fn main() {
    let mut value = INITIAL_COUNTER;
    let _updated = update_counter(&mut value);
}
