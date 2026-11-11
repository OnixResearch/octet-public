struct DomainValue;

impl DomainValue {
    fn new() -> Self {
        Self
    }
}

fn construct_local_domain_value() {
    let _value = DomainValue::new();
}

fn construct_standard_value() {
    let _values = Vec::<u8>::new();
}

fn construct_nested_external_value() {
    let _value = Some(serde_json::Map::<String, serde_json::Value>::new());
}

#[allow(explicit_defaults)]
fn construct_external_default_value() {
    let _value = serde_json::Value::default();
}

fn construct_fallible_external_value() -> Option<()> {
    let _finder = memchr::arch::all::shiftor::Finder::new(b"core")?;
    Some(())
}

fn construct_fallible_standard_value() -> Option<()> {
    let _value = std::num::NonZeroU8::new(1)?;
    Some(())
}

fn main() {
    construct_local_domain_value();
    construct_standard_value();
    construct_nested_external_value();
    construct_external_default_value();
    let _ = construct_fallible_external_value();
    let _ = construct_fallible_standard_value();
}
