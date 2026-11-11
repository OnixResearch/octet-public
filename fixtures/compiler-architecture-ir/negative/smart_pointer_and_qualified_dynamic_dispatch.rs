use std::io::Write;
use std::rc::Rc;
use std::sync::Arc;

pub trait RuntimePort {
    fn invoke(&self);
}

pub struct StaticPort;

impl RuntimePort for StaticPort {
    fn invoke(&self) {}
}

pub fn invoke_boxed(port: Box<dyn RuntimePort>) {
    port.invoke();
}

pub fn invoke_shared(port: Arc<dyn RuntimePort>) {
    port.invoke();
}

pub fn invoke_counted(port: Rc<dyn RuntimePort>) {
    port.invoke();
}

pub fn invoke_qualified(port: &dyn RuntimePort) {
    RuntimePort::invoke(port);
}

pub fn invoke_fully_qualified(port: &dyn RuntimePort) {
    <dyn RuntimePort as RuntimePort>::invoke(port);
}

pub fn invoke_static(port: Box<StaticPort>) {
    port.invoke();
    RuntimePort::invoke(&*port);
}

pub fn invoke_generic<P: RuntimePort>(port: &P) {
    port.invoke();
}

pub fn emit_boxed_writer(out: &mut Box<dyn Write>) {
    out.write_all(b"x").ok();
}

pub fn next_boxed_iterator(values: &mut Box<dyn Iterator<Item = u32>>) -> Option<u32> {
    values.next()
}

pub fn emit_boxed_buffer(out: &mut Box<Vec<u8>>) {
    out.write_all(b"x").ok();
}

pub fn next_boxed_range(values: &mut Box<std::ops::Range<u32>>) -> Option<u32> {
    values.next()
}

pub fn clone_shared(port: &Arc<dyn RuntimePort>) -> Arc<dyn RuntimePort> {
    port.clone()
}
