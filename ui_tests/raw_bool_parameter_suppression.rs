#![feature(register_tool)]
#![register_tool(tigerstyle)]
#![allow(unfulfilled_lint_expectations)]
#[allow(tigerstyle::raw_bool_parameter)]
pub fn hidden_api(enabled: bool) {
    let _ = enabled;
}

#[expect(tigerstyle::raw_bool_parameter, reason = "temporary")]
pub fn undocumented_api(enabled: bool) {
    let _ = enabled;
}

#[expect(tigerstyle::raw_bool_parameter, reason = "external compatibility contract: legacy callback ABI")]
pub fn unowned_api(enabled: bool) {
    let _ = enabled;
}

#[cfg_attr(all(), allow(tigerstyle::raw_bool_parameter))]
pub fn configured_api(enabled: bool) {
    let _ = enabled;
}

#[allow(tigerstyle::raw_bool_parameter /* comment is not part of the lint path */)]
pub fn commented_api(enabled: bool) {
    let _ = enabled;
}

pub fn local_scope() {
    #[allow(tigerstyle::raw_bool_parameter)]
    {
        let _closure = |enabled: bool, audit: bool| enabled && audit;
    }
}

#[expect(tigerstyle::raw_bool_parameter, reason = "external compatibility contract: old ABI; owner: api-team")]
mod broad_scope {
    pub fn legacy(enabled: bool) {
        let _ = enabled;
    }
}

fn main() {
    hidden_api(true);
    undocumented_api(true);
    unowned_api(true);
    configured_api(true);
    commented_api(true);
    local_scope();
    broad_scope::legacy(true);
}
