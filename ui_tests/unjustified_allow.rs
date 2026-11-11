#![feature(register_tool)]
#![register_tool(tigerstyle)]
#![warn(unjustified_allow)]
#![allow(clippy::all)]

// Should NOT warn: a crate-level allow with a justification comment.
#![allow(clippy::pedantic)]
#![allow(path_segment_repetition, reason = "fixture targets another lint; the import and path-naming lints have their own fixtures")]

#[path = "unjustified_allow_clippy/src/path_module.rs"]
mod path_module;

#[allow(clippy::needless_return)]
fn bare_allow() -> u32 {
    return 1;
}

// Should NOT warn: has justification comment
// Verified casts: Verus checks every narrowing in this module.
#[allow(clippy::cast_possible_truncation)]
fn justified_allow(x: u64) -> u32 {
    x as u32
}

fn main() {
    bare_allow();
    justified_allow(1);
}

struct Counter;

impl Counter {
    #[allow(clippy::needless_return)]
    fn bare_method_allow(&self) -> u32 {
        return 1;
    }

    // Should NOT warn: has justification comment
    // Early return mirrors the trait contract documented on `Allowed`.
    #[allow(clippy::needless_return)]
    fn justified_method_allow(&self) -> u32 {
        return 2;
    }
}

trait Allowed {
    #[allow(clippy::needless_return)]
    fn bare_default_allow(&self) -> u32 {
        return 1;
    }
}

// Should NOT warn: `reason = "..."` carries its own justification. The blank
// line keeps this fixture note from counting as a nearby `//` justification.

#[allow(clippy::needless_return, reason = "early return mirrors the protocol state table")]
fn reasoned_allow() -> u32 {
    return 3;
}

// Should warn: an empty reason is not a justification.

#[allow(clippy::needless_return, reason = "  ")]
fn empty_reason_allow() -> u32 {
    return 4;
}

impl Counter {
    // Should NOT warn: `reason = "..."` on a method allow.

    #[allow(
        clippy::needless_return,
        reason = "early return keeps the branch shape aligned with `bare_method_allow`"
    )]
    fn reasoned_method_allow(&self) -> u32 {
        return 5;
    }
}

// Should warn: Octet tool lints, bare compatibility aliases, and Octet groups
// are suppressions too. The blank lines keep these notes from justifying.

#[allow(tigerstyle::no_unwrap)]
fn tool_lint_allow() {}

#[allow(no_unwrap)]
fn bare_alias_allow() {}

#[allow(tigerstyle::safety)]
fn group_allow() {}

#[expect(clippy::needless_return)]
fn bare_expect() -> u32 {
    return 6;
}

// Should NOT warn: an `expect` with a justification comment.
// Early return mirrors the protocol state table.
#[expect(clippy::needless_return)]
fn justified_expect() -> u32 {
    return 7;
}

// Should NOT warn: rustc's own lints are outside this lint's scope.

#[allow(dead_code)]
fn rustc_lint_allow() {}

fn statement_allow() -> u32 {
    #[allow(clippy::let_and_return)]
    let value = 8;
    value
}

struct FieldAllow {
    #[allow(clippy::struct_field_names)]
    field_allow: u8,
}

enum VariantAllow {
    #[allow(clippy::large_enum_variant)]
    Large([u8; 64]),
}

// Should warn: a doc comment documents the item, not the suppression.

/// Returns a constant.
#[allow(clippy::needless_return)]
fn doc_comment_allow() -> u32 {
    return 9;
}

// Should warn: the placeholder this lint inserts is not a justification.

// TODO(octet): explain why this allow is safe here
#[allow(clippy::needless_return)]
fn placeholder_allow() -> u32 {
    return 10;
}

// Should NOT warn: a trailing comment on the attribute line justifies it.

#[allow(clippy::needless_return)] // Early return mirrors the protocol state table.
fn trailing_comment_allow() -> u32 {
    return 11;
}

// Should warn: a trailing placeholder is not a justification either.

#[allow(clippy::needless_return)] // TODO(octet): explain why this allow is safe here
fn trailing_placeholder_allow() -> u32 {
    return 12;
}

// Should warn: a `cfg_attr`-expanded allow is still a suppression the user wrote.

#[cfg_attr(all(), allow(clippy::needless_return))]
fn cfg_attr_allow() -> u32 {
    return 13;
}

// Should NOT warn: a justified `cfg_attr`-expanded allow.
// Early return mirrors the protocol state table.
#[cfg_attr(all(), allow(clippy::needless_return))]
fn justified_cfg_attr_allow() -> u32 {
    return 14;
}

// Should warn: a local `macro_rules!` expands this allow, but the user wrote it.
macro_rules! quiet {
    ($item:item) => {
        #[allow(tigerstyle::no_unwrap)]
        $item
    };
}

quiet! {
    fn macro_allow(value: Option<u8>) -> u8 {
        value.unwrap()
    }
}

// Should NOT warn: a justified allow inside a local `macro_rules!`.
macro_rules! justified_quiet {
    ($item:item) => {
        // Fixture input: callers always pass `Some`.
        #[allow(tigerstyle::no_unwrap)]
        $item
    };
}

justified_quiet! {
    fn justified_macro_allow(value: Option<u8>) -> u8 {
        value.unwrap()
    }
}

// Should warn: an allow forwarded through a local macro's `meta` fragment is
// still the user's own suppression.
macro_rules! forward_meta {
    ($(#[$meta:meta])* $name:ident { $($item:item)* }) => {
        $(#[$meta])*
        mod $name {
            $($item)*
        }
    };
}

forward_meta! {
    #[allow(clippy::needless_return)]
    forwarded_meta {
        pub fn early() -> u32 {
            return 16;
        }
    }
}

// Should NOT warn: a justified allow forwarded through a `meta` fragment.
forward_meta! {
    // Early return mirrors the protocol state table.
    #[allow(clippy::needless_return)]
    justified_forwarded_meta {
        pub fn early() -> u32 {
            return 17;
        }
    }
}

// Should warn: the attribute name forwarded as an `ident` fragment.
macro_rules! forward_level {
    ($level:ident $item:item) => {
        #[$level(tigerstyle::no_unwrap)]
        $item
    };
}

forward_level! {
    allow
    fn forwarded_level_allow(value: Option<u8>) -> u8 {
        value.unwrap()
    }
}

// Should warn: a comment inside the attribute does not hide it.

#[/**/allow(clippy::needless_return)]
fn comment_padded_allow() -> u32 {
    return 15;
}

// Should NOT warn again: the allow in `quiet!` is reported once, at its
// definition, however often the macro expands.
quiet! {
    fn second_macro_allow(value: Option<u8>) -> u8 {
        value.unwrap()
    }
}

// Should warn at the definition: expansions of an exported macro land in
// other crates, so its body is checked where it is defined, even if never
// invoked here.
#[macro_export]
macro_rules! exported_quiet {
    ($item:item) => {
        #[allow(tigerstyle::no_unwrap)]
        $item
    };
}

// Should NOT warn: a justified allow in an exported macro body.
#[macro_export]
macro_rules! exported_justified {
    ($item:item) => {
        // Callers only wrap items whose unwraps a bounds check precedes.
        #[allow(tigerstyle::no_unwrap)]
        $item
    };
}

// Should warn at the definition: a `cfg_attr` in an exported macro body
// expands into the same allow.
#[macro_export]
macro_rules! exported_cfg_attr_quiet {
    ($item:item) => {
        #[cfg_attr(all(), allow(tigerstyle::no_unwrap))]
        $item
    };
}

// Should NOT warn: a justified `cfg_attr` allow in an exported macro body.
#[macro_export]
macro_rules! exported_cfg_attr_justified {
    ($item:item) => {
        // Callers only wrap items whose unwraps a bounds check precedes.
        #[cfg_attr(all(), allow(tigerstyle::no_unwrap))]
        $item
    };
}
