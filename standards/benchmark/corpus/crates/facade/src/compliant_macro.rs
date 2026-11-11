//! A compliant macro expansion that calls only the core-owned port method.

use core_api::Port as _;

struct LocalPort;

impl core_api::Port for LocalPort {
    fn run(&self) -> u32 {
        3
    }
}

macro_rules! call_port {
    ($port:expr) => {
        $port.run()
    };
}

pub fn invoke() -> u32 {
    let port = LocalPort;
    call_port!(&port)
}
