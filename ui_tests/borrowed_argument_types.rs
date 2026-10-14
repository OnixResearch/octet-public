#![allow(improper_ctypes_definitions)]

use std::path::Path;
use std::path::PathBuf;

fn parse_name(name: &String) -> usize {
    name.len()
}

fn checksum(bytes: &Vec<u8>) -> u32 {
    bytes.iter().map(|byte| u32::from(*byte)).sum()
}

fn load_config(path: &PathBuf) -> bool {
    path.as_path().is_absolute()
}

fn checksum_qualified(bytes: &std::vec::Vec<u16>) -> u16 {
    bytes.iter().copied().sum()
}

fn already_borrowed(name: &str, bytes: &[u8], path: &Path) -> usize {
    name.len() + bytes.len() + usize::from(path.is_absolute())
}

trait Visitor {
    fn visit(&self, value: &String);
}

struct NameVisitor;

impl Visitor for NameVisitor {
    fn visit(&self, value: &String) {
        let _ = value.len();
    }
}

extern "C" fn external_callback(value: &String) {
    let _ = value.len();
}

fn main() {
    let name = String::from("tiger");
    let bytes = vec![1, 2, 3];
    let path = PathBuf::from("config.toml");
    let visitor = NameVisitor;

    let _ = parse_name(&name);
    let _ = checksum(&bytes);
    let _ = load_config(&path);
    let _ = checksum_qualified(&vec![1_u16, 2]);
    let _ = already_borrowed(&name, &bytes, path.as_path());
    visitor.visit(&name);
    external_callback(&name);
}
