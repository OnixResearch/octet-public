#![feature(register_tool)]
#![register_tool(tigerstyle)]
#![allow(warnings)]
#![allow(tigerstyle::structure)]
#![allow(tigerstyle::raw_bool_parameter_suppression)]
#![allow(tigerstyle::raw_bool_parameter)]

pub fn legacy_api(enabled: bool) {
    let _ = enabled;
}

#[allow(tigerstyle::structure)]
mod broad_scope {
    pub fn hidden_api(enabled: bool) {
        let _ = enabled;
    }
}

fn arm_scopes(case: u8) {
    match case {
        #[allow(tigerstyle::raw_bool_parameter)]
        0 => {
            let callback = |first: bool, second: bool| first && second;
            let _ = callback(true, false);
        }
        #[cfg_attr(all(), allow(tigerstyle::structure))]
        1 => {
            let callback = |first: bool, second: bool| first || second;
            let _ = callback(false, true);
        }
        #[allow(dead_code)]
        _ => {}
    }
}

fn main() {
    legacy_api(true);
    broad_scope::hidden_api(false);
    arm_scopes(0);
}
