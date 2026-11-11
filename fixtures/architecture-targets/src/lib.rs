architecture_target_macro::identity! {
    pub fn pure(value: bool) -> bool {
        value
    }
}

pub trait Associated {
    type Value;
    const PRESENT: bool;

    fn identity(value: Self::Value) -> Self::Value {
        value
    }
}

pub struct AssociatedValue;

impl Associated for AssociatedValue {
    type Value = bool;
    const PRESENT: bool = true;
}

#[cfg(feature = "extra")]
pub fn extra_effect() {
    let _ = std::fs::read("never-executed");
}

#[cfg(test)]
#[test]
fn test_effect() {
    let _ = std::fs::read("never-executed");
}
