fn uses_unwrap() {
    // Should warn: unwrap on Result
    let val: Result<u32, &str> = Ok(1);
    let _ = val.unwrap();
}

fn uses_expect() {
    // Should warn: expect on Option
    let val: Option<u32> = Some(1);
    let _ = val.expect("missing");
}

fn uses_question_mark() -> Result<u32, &'static str> {
    // Should NOT warn: proper error propagation
    let val: Result<u32, &str> = Ok(1);
    let v = val?;
    Ok(v)
}

fn main() {
    uses_unwrap();
    uses_expect();
}

#[test]
fn unwrap_in_test_is_allowed() {
    let val: Option<u32> = Some(1);
    let _ = val.unwrap();
}

#[cfg(test)]
mod cfg_test_only {
    pub(super) fn unwrap_in_cfg_test_module_is_allowed() {
        let val: Option<u32> = Some(1);
        let _ = val.unwrap();
    }
}
