#![allow(improper_ctypes_definitions, missing_abi)]
#![allow(non_trait_imports, reason = "fixture targets another lint; the import and path-naming lints have their own fixtures")]

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

// Should NOT warn: mutable references need the owned container's growth API.
fn fill(out: &mut Vec<u8>) {
    out.push(1);
}

fn append_suffix(name: &mut String) {
    name.push('!');
}

fn extend_path(path: &mut PathBuf) {
    path.push("child");
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

// Should NOT warn: `extern fn` without an ABI string is a C-ABI function.
extern fn external_default_abi(value: &String) {
    let _ = value.len();
}

// Should warn: an `extern "C" fn` parameter type does not make this Rust-ABI function extern.
fn register_callback(name: &String, callback: extern "C" fn()) -> usize {
    callback();
    name.len()
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
    let mut buffer = Vec::new();
    fill(&mut buffer);
    let mut label = String::new();
    append_suffix(&mut label);
    let mut root = PathBuf::new();
    extend_path(&mut root);
    visitor.visit(&name);
    external_callback(&name);
}
