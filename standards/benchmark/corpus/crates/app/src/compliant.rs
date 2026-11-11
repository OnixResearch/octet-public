//! A compliant application-shell call through the core-owned port.

pub fn dispatch(port: &dyn core_api::Port) -> u32 {
    port.run()
}
