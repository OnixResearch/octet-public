//! A compliant public alias of the core-owned port.

pub use core_api::Port as ServicePort;

pub fn dispatch(port: &dyn ServicePort) -> u32 {
    port.run()
}
