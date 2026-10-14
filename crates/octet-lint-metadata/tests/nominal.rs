use serde::Deserialize;
use serde::Serialize;

const GOLDEN_WIRE_JSON: &str = r#"{"unit":"unit-a","source":"source-a"}"#;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct WireRecord {
    unit: String,
    source: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct UnitId(String);

#[derive(Clone, Debug, Eq, PartialEq)]
struct SourceId(String);

#[derive(Clone, Debug, Eq, PartialEq)]
struct TypedRecord {
    unit: UnitId,
    source: SourceId,
}

fn admit_non_empty(value: String, field: &'static str) -> Result<String, &'static str> {
    if value.is_empty() {
        return Err(field);
    }
    Ok(value)
}

fn admit_wire(wire: WireRecord) -> Result<TypedRecord, &'static str> {
    Ok(TypedRecord {
        unit: UnitId(admit_non_empty(wire.unit, "unit")?),
        source: SourceId(admit_non_empty(wire.source, "source")?),
    })
}

fn project_wire(typed: TypedRecord) -> WireRecord {
    WireRecord {
        unit: typed.unit.0,
        source: typed.source.0,
    }
}

#[test]
fn typed_admission_preserves_the_wire_projection() {
    let wire: WireRecord = serde_json::from_str(GOLDEN_WIRE_JSON).expect("valid wire fixture");
    let typed = admit_wire(wire).expect("admitted wire fixture");
    let projected = serde_json::to_string(&project_wire(typed)).expect("serialize projected wire fixture");

    assert_eq!(projected, GOLDEN_WIRE_JSON);
}

#[test]
fn typed_admission_rejects_malformed_wire_values() {
    let malformed = WireRecord {
        unit: String::new(),
        source: String::from("source-a"),
    };

    assert_eq!(admit_wire(malformed), Err("unit"));
}
