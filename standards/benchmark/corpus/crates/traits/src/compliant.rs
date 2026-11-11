//! A compliant trait-object method reference.

pub fn call_via_trait(port: &dyn core_api::Port) -> u32 {
    core_api::Port::run(port)
}
