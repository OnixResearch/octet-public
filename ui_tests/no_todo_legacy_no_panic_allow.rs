#![feature(register_tool)]
#![register_tool(tigerstyle)]

// legacy no_panic escape no longer suppresses todo!
#[allow(tigerstyle::no_panic)]
fn tool_scoped_legacy_no_panic_attr() {
    todo!("legacy tool-scoped no_panic attr should not suppress no_todo");
}

// legacy bare no_panic escape no longer suppresses todo!
#[allow(no_panic)]
fn bare_legacy_no_panic_attr() {
    todo!("legacy bare no_panic attr should not suppress no_todo");
}

fn main() {
    tool_scoped_legacy_no_panic_attr();
    bare_legacy_no_panic_attr();
}
