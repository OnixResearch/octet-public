// Without a strict profile, `warn` is the normal way to enable lints and
// stays ungoverned, even for a lint a profile would require.
#![warn(no_unwrap)]
#![deny(unjustified_allow)]

fn main() {}
