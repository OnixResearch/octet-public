// A selected strict profile keeps its suppression-governance lints at
// `deny`, and no source `allow`, `expect`, or `warn` may reach them, even
// with a justification: such an attribute would silence its own audit.
#![feature(register_tool)]
#![register_tool(tigerstyle)]
#![allow(tigerstyle::structure)]

// Should error: `tigerstyle::structure` contains `unjustified_allow`.
#[allow(tigerstyle::structure)]
mod grouped {
    #[allow(clippy::needless_return)]
    pub fn early() -> u32 {
        return 1;
    }
}

// Should error: a justification does not make the group allow legal.
// The structure lints are noisy in this generated module.
#[allow(tigerstyle::structure)]
mod justified_group {}

// Should error: the bare alias names the audit lint itself.
#[allow(unjustified_allow)]
mod bare {}

// Should error: the canonical name of the audit lint.
#[expect(tigerstyle::unjustified_allow)]
mod expected {}

// Should error: `warn` would lower the `no_todo` audit below the floor.
#[warn(unjustified_no_todo_allow)]
mod warned {}

// Should error: `tigerstyle::safety` contains `unjustified_no_todo_allow`.
#[allow(tigerstyle::safety)]
fn quiet() {}

macro_rules! module {
    ($(#[$meta:meta])* $name:ident { $($item:item)* }) => {
        $(#[$meta])*
        mod $name {
            $($item)*
        }
    };
}

macro_rules! leveled {
    ($level:ident $name:ident) => {
        #[$level(tigerstyle::structure)]
        mod $name {}
    };
}

// Should error: the allow is forwarded through a local macro's `meta` fragment.
module!(#[allow(tigerstyle::structure)] forwarded {});

// Should error: the attribute name is forwarded as an `ident` fragment.
leveled!(allow forwarded_level);

// Should error: a comment inside the attribute does not hide it.
#[/**/allow(tigerstyle::structure)]
mod padded {}

macro_rules! silenced {
    ($name:ident) => {
        // Should error once, at this definition, though the macro expands twice.
        #[allow(tigerstyle::structure)]
        mod $name {}
    };
}

silenced!(first_silenced);
silenced!(second_silenced);

fn main() {
    let _ = grouped::early();
    quiet();
}

// Should error: the later `deny` raises `no_unwrap` back, but not
// `unjustified_no_todo_allow`.
#[warn(tigerstyle::safety)]
#[deny(tigerstyle::no_unwrap)]
mod partly_restored {}

// Should error: a `deny` before the `warn` does not raise it back.
#[deny(tigerstyle::safety)]
#[warn(tigerstyle::safety)]
mod lowered_last {}

// Should error at the definition, though the exported macro is never invoked
// here: its expansions in other crates would silence the audit.
#[macro_export]
macro_rules! exported_silenced {
    () => {
        #[allow(tigerstyle::structure)]
        mod exported_silenced {}
    };
}

// Should error at the definition: `cfg_attr` would expand into the allow.
#[macro_export]
macro_rules! exported_cfg_attr_silenced {
    () => {
        #[cfg_attr(all(), allow(tigerstyle::structure))]
        mod exported_cfg_attr_silenced {}
    };
}

// Should error: a doc comment does not hide the allow.
#[macro_export]
macro_rules! exported_documented_silenced {
    () => {
        #[allow(tigerstyle::structure)]
        /// Generated helper.
        fn documented() {}
    };
}

macro_rules! repeated_attributes {
    ($(#[$meta:meta])* $name:ident) => {
        #[warn(tigerstyle::safety)]
        $(#[$meta])*
        mod $name {}
    };
}

// Should error at the expansion: this invocation adds no later raise.
repeated_attributes!(lowered_by_invocation);
