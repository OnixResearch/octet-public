#![warn(type_accumulation)]

// The crate root holds exactly the default limit of 12 public items. The fixture
// compiles with `--test`, and the harness adds a public `main` plus a public
// `#[rustc_test_marker]` const per test; neither is public surface the author wrote.
pub struct Thing;

impl Thing { pub fn a(&self) {} }
impl Thing { pub fn b(&self) {} }

pub fn item_01() {}
pub fn item_02() {}
pub fn item_03() {}
pub fn item_04() {}
pub fn item_05() {}
pub fn item_06() {}
pub fn item_07() {}
pub fn item_08() {}
pub fn item_09() {}
pub fn item_10() {}
pub fn item_11() {}

#[test]
fn thing_methods_run() {
    Thing.a();
    Thing.b();
}

fn main() {}
