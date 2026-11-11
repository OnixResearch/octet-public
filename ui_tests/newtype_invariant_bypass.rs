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

// A second registered `CheckedToken` whose bypasses go through borrowed views
// of the `String` representation.
mod borrowed_views {
    use std::ops::Deref;

    pub struct CheckedToken(String);

    // Should warn: `From<&str>` copies unchecked borrowed input into raw storage.
    impl From<&str> for CheckedToken {
        fn from(value: &str) -> Self {
            Self(value.to_owned())
        }
    }

    // Should warn: `Target = str` exposes the raw storage's borrowed view.
    impl Deref for CheckedToken {
        type Target = str;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
}

fn main() {
    let token = CheckedToken::from(String::new());
    let _ = token.len();
    let view = borrowed_views::CheckedToken::from("token");
    let _ = view.len();
}
