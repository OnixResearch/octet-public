#![no_std]

pub fn read_home() -> Option<std::string::String> {
    std::env::var("HOME").ok()
}
