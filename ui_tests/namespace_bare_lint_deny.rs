#![deny(no_unwrap)]

fn bare_lint_alias_behaves_like_canonical_lint() {
    let value: Option<u32> = Some(1);
    let _ = value.unwrap();
}

fn main() {
    bare_lint_alias_behaves_like_canonical_lint();
}
