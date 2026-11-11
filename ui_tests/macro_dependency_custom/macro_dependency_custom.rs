// Under a strict profile, a `macro_rules!` helper defined by expanding
// another crate's macro is that crate's code: its body is neither rejected
// by the governance guard nor reported by `unjustified_allow` here.
#![deny(unjustified_allow)]

governance_dep::define_helper!();

fn main() {}
