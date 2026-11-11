#![feature(register_tool)]
#![register_tool(tigerstyle)]

#[expect(
    tigerstyle::raw_bool_parameter,
    reason = "external compatibility contract: caller-owned legacy wire ABI; owner: api-team"
)]
pub fn legacy_wire_api(enabled: bool) {
    let _ = enabled;
}

fn main() {
    legacy_wire_api(true);
}
