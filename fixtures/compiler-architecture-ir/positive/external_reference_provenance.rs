use std::path::PathBuf;

pub fn publish(_target: &PathBuf) {}

pub fn execute(target: &PathBuf) {
    publish(target);
}
