//! A compliant composition-root binary that wires the port explicitly.

struct HostPort;

impl core_api::Port for HostPort {
    fn run(&self) -> u32 {
        5
    }
}

fn dispatch(port: &dyn core_api::Port) -> u32 {
    port.run()
}

fn main() {
    let port = HostPort;
    let _ = dispatch(&port);
}
