#![no_std]

pub fn ambient_process_count() -> usize {
    std::process::id() as usize
}
