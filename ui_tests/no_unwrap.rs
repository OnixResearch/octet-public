#![allow(path_segment_repetition, reason = "fixture targets another lint; the import and path-naming lints have their own fixtures")]

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

fn unwrap_through_option_reference(value: &Option<u32>) -> u32 {
    // Should warn: unwrap on &Option auto-derefs
    value.unwrap()
}

fn expect_through_result_reference(value: &Result<u32, &'static str>) -> u32 {
    // Should warn: expect on &Result auto-derefs
    value.expect("missing")
}

fn unwrap_in_closure_over_option_slice(values: &[Option<u32>]) -> Vec<u32> {
    // Should warn: closure receives &Option<u32>
    values.iter().map(|value| value.unwrap()).collect()
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

struct CfgTestMethodHost;

impl CfgTestMethodHost {
    #[cfg(test)]
    fn unwrap_in_cfg_test_impl_method_is_allowed() {
        let val: Result<u32, &str> = Ok(1);
        let _ = val.unwrap();
    }
}

trait CfgTestTraitHost {
    #[cfg(test)]
    fn unwrap_in_cfg_test_trait_method_is_allowed() {
        let val: Result<u32, &str> = Ok(1);
        let _ = val.unwrap();
    }
}

#[cfg(test)]
#[derive(Debug)] struct OneLineTestProbe;

// The `#[cfg(test)]` above belongs to the one-line item, not to this function.
fn unwrap_below_one_line_test_item_is_reported() -> u32 {
    let val: Option<u32> = Some(1);
    val.unwrap()
}

fn unwrap_path_in_map(values: Vec<Result<u32, &'static str>>) -> Vec<u32> {
    // Should warn: `Result::unwrap` passed as a function path
    values.into_iter().map(Result::unwrap).collect()
}

fn expect_path_in_map(values: Vec<Option<u32>>) -> Vec<u32> {
    // Should warn: `Option::expect` passed as a function path
    values.into_iter().map(|value| Option::expect(value, "missing")).collect()
}

fn ufcs_result_unwrap(value: Result<u32, &'static str>) -> u32 {
    // Should warn: UFCS `Result::unwrap` call
    Result::unwrap(value)
}

fn qualified_option_unwrap(value: Option<u32>) -> u32 {
    // Should warn: qualified-self `<Option<u32>>::unwrap` call
    <Option<u32>>::unwrap(value)
}

fn ufcs_unwrap_or_is_allowed(value: Option<u32>) -> u32 {
    // Should NOT warn: `unwrap_or` does not panic
    Option::unwrap_or(value, 0)
}

fn ufcs_constructor_unwrap_is_allowed() -> u32 {
    // Should NOT warn: known constructor, as with `Some(1).unwrap()`
    Option::unwrap(Some(1))
}

#[test]
fn ufcs_unwrap_in_test_is_allowed() {
    let values: Vec<Option<u32>> = vec![Some(1)];
    let _ = values.into_iter().map(Option::unwrap).count();
    let _ = Result::<u32, &str>::unwrap(Ok(1));
}
