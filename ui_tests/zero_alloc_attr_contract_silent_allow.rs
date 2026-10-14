// zero_alloc_attr_contract_silent_allow: bare allows without justification
// trigger unjustified_allow

#![feature(register_tool)]
#![register_tool(tigerstyle)]
#![deny(unjustified_allow)]

#[allow(tigerstyle::zero_alloc_attr_contract)]
fn silent_allow() {
    let _v: Vec<u8> = Vec::new();
}

fn main() {
    silent_allow();
}