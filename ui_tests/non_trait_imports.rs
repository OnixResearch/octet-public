#![allow(dead_code, unused_imports)]

mod addr {
    pub struct Addr;
}

mod rustc_hir {
    pub struct Item;
}

// Should NOT warn: canonical rustc-private shorthand keeps lint implementation code readable.
use crate::rustc_hir as hir;

use crate::addr::Addr;

fn route(addr: Addr) {
    let _ = addr;
}

fn use_rustc_alias(_: hir::Item) {}

fn main() {}
