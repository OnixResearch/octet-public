// trait FakePort { fn fake(&self); }
const SOURCE_TEXT_ONLY: &str = "impl FakePort for FakeAdapter";

pub trait RuntimePort {
    fn invoke(&self);
}

pub fn invoke_dynamic(port: &dyn RuntimePort) {
    port.invoke();
}

pub fn indirect_factory() -> Box<dyn RuntimePort> {
    struct Adapter;
    impl RuntimePort for Adapter {
        fn invoke(&self) {}
    }
    Box::new(Adapter)
}

#[cfg(any())]
impl RuntimePort for String {
    fn invoke(&self) {}
}

pub fn retained_text() -> &'static str {
    SOURCE_TEXT_ONLY
}
