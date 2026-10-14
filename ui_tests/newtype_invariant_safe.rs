#![warn(newtype_invariant_bypass)]

#[derive(Debug)]
pub struct CheckedToken(String);

impl TryFrom<String> for CheckedToken {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err("token must not be empty");
        }
        Ok(Self(value))
    }
}

impl CheckedToken {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn main() {
    let Ok(token) = CheckedToken::try_from(String::from("valid-token")) else {
        return;
    };
    let _ = token.as_str();
}
