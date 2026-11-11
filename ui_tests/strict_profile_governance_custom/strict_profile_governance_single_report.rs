#![feature(register_tool)]
#![register_tool(tigerstyle)]

// Under a strict profile an attribute that reaches a suppression-governance
// lint gets only the guard's error: an `unjustified_allow` finding asking for
// a justification could never clear it. No crate-level allow masks this file.

// Should get only the guard error: `tigerstyle::safety` contains
// `unjustified_no_todo_allow`.

#[allow(tigerstyle::safety)]
fn grouped() {}

// Should get only the guard error: the governance lint by name.

#[allow(unjustified_no_todo_allow)]
fn named() {}

// Should get only the guard error: `warn` of a group with governance and
// floor members.

#[warn(tigerstyle::safety)]
fn warned() {}

// Should get only the `unjustified_allow` finding: no governance lint.

#[allow(tigerstyle::no_unwrap)]
fn ordinary() {}

fn main() {
    grouped();
    named();
    warned();
    ordinary();
}
