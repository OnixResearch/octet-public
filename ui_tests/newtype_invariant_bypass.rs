#![warn(newtype_invariant_bypass)]

use std::ops::Deref;

// Should warn: the raw field is public.
pub struct CheckedToken(pub String);

// Should warn: `From<String>` bypasses validation.
impl From<String> for CheckedToken {
    fn from(value: String) -> Self {
        Self(value)
    }
}

// Should warn: raw dereference erases the boundary.
impl Deref for CheckedToken {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn main() {
    let token = CheckedToken::from(String::new());
    let _ = token.len();
}
