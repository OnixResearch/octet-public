use std::time::Instant;

pub fn deadline_or_now(deadline: Option<Instant>) -> Instant {
    deadline.unwrap_or_else(Instant::now)
}

pub fn read_all(paths: Vec<String>) -> Vec<std::io::Result<String>> {
    paths.into_iter().map(std::fs::read_to_string).collect()
}

pub fn closure_now(deadline: Option<Instant>) -> Instant {
    deadline.unwrap_or_else(|| Instant::now())
}

fn increment(value: u32) -> u32 {
    value + 1
}

pub fn pure_reference(values: Vec<u32>) -> Vec<u32> {
    values.into_iter().map(increment).collect()
}
