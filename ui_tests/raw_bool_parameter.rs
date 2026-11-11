#![feature(register_tool)]
#![register_tool(tigerstyle)]

pub fn deploy(enabled: bool, mode: bool) {
    let _ = (enabled, mode);
}

mod reexported {
    pub fn via_private_module(enabled: bool) {
        let _ = enabled;
    }
}
pub use reexported::via_private_module;

pub fn co_report(first: bool, second: bool, third: bool, fourth: bool) {
    let _ = (first, second, third, fourth);
}

pub struct Controller;

impl Controller {
    pub fn set_mode(&self, enabled: bool) {
        let _ = enabled;
    }

    fn configure(&self, enabled: bool, audit: bool) {
        let _ = (enabled, audit);
    }
}

pub trait PublicControl {
    fn toggle(&self, enabled: bool);
}

trait PrivateControl {
    fn configure(&self, enabled: bool, audit: bool);
}

fn private_flags(enabled: bool, audit: bool) {
    let _ = (enabled, audit);
}

fn main() {
    private_flags(false, true);
    via_private_module(true);
    co_report(true, false, true, false);
    Controller.configure(true, false);
    let configure = |enabled: bool, audit: bool| {
        let _ = (enabled, audit);
    };
    configure(true, false);
}
