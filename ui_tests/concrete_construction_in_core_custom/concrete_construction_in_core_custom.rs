#![allow(non_trait_imports, reason = "fixture targets alias resolution in the construction lint")]

fn construct_reviewed_value() {
    let _value = serde_json::Map::<String, serde_json::Value>::new();
}

fn construct_aliased_reviewed_value() {
    use serde_json::Map as ValueMap;
    let _value = ValueMap::<String, serde_json::Value>::new();
}

fn construct_unreviewed_sibling() {
    let _value = serde_json::Value::default();
}

fn main() {
    construct_reviewed_value();
    construct_aliased_reviewed_value();
    construct_unreviewed_sibling();
}
