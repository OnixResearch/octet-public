use std::{error::Error, fs, path::PathBuf, process::Command};

use cairn_core::{
    evaluate_function_address_stack_smoke, FunctionAddressStackSmokeInput,
    FunctionAddressStackSmokeReport,
};
use serde_json::{json, Value};

type FixtureMutation = Box<dyn Fn(&mut Fixture)>;

struct Fixture {
    source: Vec<u8>,
    expected: Value,
    octet: Value,
    kamacite: Value,
    valence: Value,
    mantle: Value,
    graph: Value,
}

impl Fixture {
    fn evaluate(&self) -> FunctionAddressStackSmokeReport {
        evaluate_function_address_stack_smoke(&FunctionAddressStackSmokeInput {
            source: &self.source,
            expected: &self.expected,
            octet: &self.octet,
            kamacite: &self.kamacite,
            valence: &self.valence,
            mantle: &self.mantle,
            graph: &self.graph,
        })
    }
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .unwrap_or_else(|| panic!("workspace root must contain crates/cairn-cli"))
        .to_path_buf()
}

fn fixture_root() -> PathBuf {
    workspace_root().join("fixtures/function-address-stack-smoke")
}

fn read_json(path: &str) -> Result<Value, Box<dyn Error>> {
    Ok(serde_json::from_slice(&fs::read(
        fixture_root().join(path),
    )?)?)
}

fn fixture() -> Result<Fixture, Box<dyn Error>> {
    Ok(Fixture {
        source: fs::read(fixture_root().join("source/src/lib.rs"))?,
        expected: read_json("expected.json")?,
        octet: read_json("positive/octet-function-addresses.json")?,
        kamacite: read_json("positive/kamacite.json")?,
        valence: read_json("positive/valence.json")?,
        mantle: read_json("positive/mantle.json")?,
        graph: read_json("positive/graph.json")?,
    })
}

#[test]
// r[verify cairn.release_evidence.function_address_stack_smoke.fixture]
// r[verify cairn.release_evidence.function_address_stack_smoke.expected_outputs]
// r[verify cairn.release_evidence.function_address_stack_smoke.first_class_commands]
// r[verify cairn.release_evidence.function_address_stack_smoke.docs]
// r[verify cairn.release_evidence.function_address_stack_smoke.validation]
fn committed_stack_smoke_fixture_passes() -> Result<(), Box<dyn Error>> {
    let fixture = fixture()?;
    let report = fixture.evaluate();
    assert!(report.valid, "{:?}", report.issues);
    assert_eq!(report.to_json()["verdict"], "PASS");

    let root = workspace_root();
    let positive = fixture_root().join("positive");
    let output = Command::new(env!("CARGO_BIN_EXE_cairn"))
        .current_dir(&root)
        .args([
            "release-readiness",
            "--root",
            ".",
            "--function-address-valence",
        ])
        .arg(positive.join("valence.json"))
        .arg("--function-address-mantle")
        .arg(positive.join("mantle.json"))
        .arg("--function-address-kamacite")
        .arg(positive.join("kamacite.json"))
        .arg("--function-address-graph")
        .arg(positive.join("graph.json"))
        .output()?;
    assert!(output.status.success());
    let receipt: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(receipt["function_address"]["passed"], true);
    assert_eq!(receipt["function_address"]["disposition"], "present");
    let expected_readiness = read_json("positive/cairn-readiness.json")?;
    assert_eq!(
        receipt["function_address"],
        expected_readiness["function_address"]
    );
    Ok(())
}

#[test]
// r[verify cairn.release_evidence.function_address_stack_smoke.negative]
fn boundary_mutations_fail_with_specific_diagnostics() -> Result<(), Box<dyn Error>> {
    let cases: Vec<(&str, FixtureMutation)> = vec![
        (
            "function_address_smoke.function_address_stale",
            Box::new(|fixture| {
                fixture.octet["records"][0]["function_address"] =
                    json!("b3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
            }),
        ),
        (
            "function_address_smoke.source_span_invalid",
            Box::new(|fixture| fixture.octet["records"][0]["end_byte"] = json!(u64::MAX)),
        ),
        (
            "function_address_smoke.mantle_valence_link_stale",
            Box::new(|fixture| {
                fixture.mantle["valence_receipt_digest"] =
                    json!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            }),
        ),
        (
            "function_address_smoke.valence_role_wrong",
            Box::new(|fixture| fixture.valence["roles"] = json!(["wrong-role"])),
        ),
        (
            "function_address_smoke.claim_scope_wrong",
            Box::new(|fixture| fixture.mantle["claim_scope"] = json!("release-eligible")),
        ),
        (
            "function_address_smoke.valence_receipt_stale",
            Box::new(|fixture| fixture.valence = Value::Null),
        ),
        (
            "function_address_smoke.non_claim_missing_or_overstated",
            Box::new(|fixture| fixture.octet["non_claims"] = json!(["proves release eligibility"])),
        ),
    ];

    for (expected_code, mutate) in cases {
        let mut fixture = fixture()?;
        mutate(&mut fixture);
        let report = fixture.evaluate();
        assert!(
            !report.valid,
            "mutation {expected_code} unexpectedly passed"
        );
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.code == expected_code),
            "missing {expected_code}: {:?}",
            report.issues
        );
    }
    Ok(())
}
