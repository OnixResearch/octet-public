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

fn main() {
    construct_local_domain_value();
    construct_standard_value();
    construct_nested_external_value();
}
