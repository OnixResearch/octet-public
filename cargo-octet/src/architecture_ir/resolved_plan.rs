//! Pure comparison of complete Cargo plans. Array indices are not semantic identities.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
struct Graph {
    units: Vec<Unit>,
    roots: Vec<usize>,
    #[serde(flatten)]
    properties: BTreeMap<String, Value>,
}

#[derive(Deserialize)]
struct Unit {
    dependencies: Vec<Dependency>,
    #[serde(flatten)]
    description: BTreeMap<String, Value>,
}

#[derive(Deserialize)]
struct Dependency {
    index: usize,
    #[serde(flatten)]
    description: BTreeMap<String, Value>,
}

// r[impl octet.architecture.features.equivalence]
// r[impl octet.architecture.features.rejection]
pub(super) fn identity(bytes: &[u8]) -> Result<String, String> {
    let graph: Graph = serde_json::from_slice(bytes).map_err(|error| format!("parse complete Cargo plan: {error}"))?;
    if graph.units.is_empty() || graph.roots.is_empty() {
        return Err("Cargo feature plan has an empty selection".to_owned());
    }
    let identities = subgraph_identities(&graph.units)?;
    let units = identities.iter().map(String::as_str).collect::<BTreeSet<_>>();
    if units.len() != identities.len() {
        return Err("Cargo feature plan has ambiguous duplicate unit identities".to_owned());
    }
    let mut roots = graph
        .roots
        .iter()
        .map(|index| referenced_identity(&identities, *index))
        .collect::<Result<Vec<_>, _>>()?;
    roots.sort_unstable();
    octet_architecture_ir::canonical_identity("octet.cargo-feature-plan/v2", &(graph.properties, units, roots))
}

/// Cargo plans one package twice with equal descriptions when only the features
/// of its dependencies differ, as resolver 2 does for a build-script copy. Each
/// identity therefore binds the unit's complete dependency subgraph, computed
/// leaves first. Units on a dependency cycle never resolve and fail closed.
fn subgraph_identities(units: &[Unit]) -> Result<Vec<String>, String> {
    let mut unresolved_dependencies = Vec::with_capacity(units.len());
    let mut dependents = vec![Vec::new(); units.len()];
    for (index, unit) in units.iter().enumerate() {
        unresolved_dependencies.push(unit.dependencies.len());
        for edge in &unit.dependencies {
            dependents
                .get_mut(edge.index)
                .ok_or_else(|| "Cargo feature plan has an invalid unit reference".to_owned())?
                .push(index);
        }
    }
    let mut ready = Vec::with_capacity(units.len());
    ready.extend((0..units.len()).filter(|index| unresolved_dependencies[*index] == 0));
    let mut identities: Vec<Option<String>> = vec![None; units.len()];
    while let Some(index) = ready.pop() {
        let unit = &units[index];
        let mut edges = Vec::with_capacity(unit.dependencies.len());
        for edge in &unit.dependencies {
            let destination = identities[edge.index]
                .as_deref()
                .ok_or_else(|| "Cargo feature plan resolved a unit before its dependency".to_owned())?;
            edges.push(octet_architecture_ir::canonical_identity(
                "octet.cargo-feature-plan.edge/v2",
                &(destination, &edge.description),
            )?);
        }
        edges.sort_unstable();
        identities[index] = Some(octet_architecture_ir::canonical_identity(
            "octet.cargo-feature-plan.subgraph/v1",
            &(unit_identity(unit)?, edges),
        )?);
        for dependent in &dependents[index] {
            unresolved_dependencies[*dependent] -= 1;
            if unresolved_dependencies[*dependent] == 0 {
                ready.push(*dependent);
            }
        }
    }
    identities
        .into_iter()
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| "Cargo feature plan has a dependency cycle".to_owned())
}

fn unit_identity(unit: &Unit) -> Result<String, String> {
    let mut description = unit.description.clone();
    let features = description
        .remove("features")
        .ok_or_else(|| "Cargo feature plan omits resolved features".to_owned())?;
    let mut features: Vec<String> =
        serde_json::from_value(features).map_err(|error| format!("invalid Cargo resolved features: {error}"))?;
    features.sort();
    features.dedup();
    octet_architecture_ir::canonical_identity("octet.cargo-feature-plan.unit/v1", &(description, features))
}

fn referenced_identity(identities: &[String], index: usize) -> Result<&str, String> {
    identities
        .get(index)
        .map(String::as_str)
        .ok_or_else(|| "Cargo feature plan has an invalid unit reference".to_owned())
}

#[cfg(test)]
mod tests {
    use serde_json::Value;
    use serde_json::json;

    fn graph() -> Value {
        json!({"version": 1, "units": [
            {"pkg_id": "app", "features": ["extra", "base"], "mode": "test",
             "dependencies": [{"index": 1, "extern_crate_name": "helper", "public": false}]},
            {"pkg_id": "helper", "features": [], "mode": "build", "dependencies": []}
        ], "roots": [0, 1]})
    }

    fn identity(value: &Value) -> Result<String, String> {
        super::identity(&serde_json::to_vec(value).expect("fixture JSON"))
    }

    // r[verify octet.architecture.features.equivalence]
    #[test]
    fn order_and_indices_do_not_define_equivalence() {
        let original = graph();
        let mut reordered = original.clone();
        reordered["units"].as_array_mut().expect("units").reverse();
        reordered["units"][1]["dependencies"][0]["index"] = json!(0);
        reordered["units"][1]["features"] = json!(["base", "extra"]);
        reordered["roots"] = json!([1, 0]);
        assert_eq!(identity(&original), identity(&reordered));
    }

    // r[verify octet.architecture.features.equivalence]
    #[test]
    fn dependency_order_does_not_define_equivalence() {
        let mut original = graph();
        original["units"][0]["dependencies"]
            .as_array_mut()
            .expect("edges")
            .push(json!({"index": 1, "extern_crate_name": "alias", "public": true}));
        let mut reordered = original.clone();
        reordered["units"][0]["dependencies"].as_array_mut().expect("edges").reverse();
        assert_eq!(identity(&original), identity(&reordered));
    }

    // r[verify octet.architecture.features.equivalence]
    #[test]
    fn changed_features_edges_roots_and_descriptions_change_identity() {
        let original = graph();
        let expected = identity(&original).expect("valid plan");
        for (pointer, replacement) in [
            ("/units/0/features", json!(["base"])),
            ("/units/0/dependencies", json!([])),
            ("/units/0/dependencies/0/extern_crate_name", json!("renamed")),
            ("/units/0/dependencies/0/public", json!(true)),
            ("/units/0/mode", json!("build")),
            ("/roots", json!([1])),
            ("/version", json!(2)),
        ] {
            let mut altered = original.clone();
            *altered.pointer_mut(pointer).expect("existing field") = replacement;
            assert_ne!(identity(&altered).expect("valid altered plan"), expected, "{pointer}");
        }
        let mut extra_field = original.clone();
        extra_field["units"][0]["future_cargo_field"] = json!("must remain bound");
        assert_ne!(identity(&extra_field).expect("extra field"), expected);

        let mut redirected = original.clone();
        redirected["units"]
            .as_array_mut()
            .expect("units")
            .push(json!({"pkg_id": "other", "features": [], "mode": "build", "dependencies": []}));
        let before_redirect = identity(&redirected).expect("plan with another leaf");
        redirected["units"][0]["dependencies"][0]["index"] = json!(2);
        assert_ne!(identity(&redirected).expect("redirected edge"), before_redirect);
    }

    // r[verify octet.architecture.features.equivalence]
    #[test]
    fn units_distinguished_only_by_dependency_subgraphs_remain_distinct() {
        // Resolver 2 plans a build-script copy of `toml` whose `serde_core` dependency
        // resolves fewer features; both `toml` units carry equal descriptions.
        let plan = json!({"version": 1, "units": [
            {"pkg_id": "toml", "features": ["std"], "mode": "build",
             "dependencies": [{"index": 2, "extern_crate_name": "serde_core", "public": false}]},
            {"pkg_id": "toml", "features": ["std"], "mode": "build",
             "dependencies": [{"index": 3, "extern_crate_name": "serde_core", "public": false}]},
            {"pkg_id": "serde_core", "features": ["std"], "mode": "build", "dependencies": []},
            {"pkg_id": "serde_core", "features": ["result", "std"], "mode": "build", "dependencies": []}
        ], "roots": [0, 1]});
        identity(&plan).expect("dependency-distinguished units are unambiguous");

        let mut transitive = plan.clone();
        transitive["units"][3]["features"] = json!(["std"]);
        transitive["units"].as_array_mut().expect("units").push(json!(
            {"pkg_id": "serde_derive", "features": [], "mode": "build", "dependencies": []}
        ));
        transitive["units"][3]["dependencies"] =
            json!([{"index": 4, "extern_crate_name": "serde_derive", "public": false}]);
        identity(&transitive).expect("units distinguished two dependency levels down are unambiguous");

        let mut collapsed = plan.clone();
        collapsed["units"][1]["dependencies"][0]["index"] = json!(2);
        assert!(identity(&collapsed).expect_err("equal subgraphs").contains("duplicate"));
    }

    // r[verify octet.architecture.features.rejection]
    #[test]
    fn incomplete_or_ambiguous_plans_fail_closed() {
        for (pointer, replacement) in [
            ("/units/0/dependencies/0/index", json!(2)),
            ("/units/0/dependencies/0/index", json!(0)),
            ("/units/1/dependencies", json!([{"index": 0, "extern_crate_name": "app", "public": false}])),
            ("/units/0/dependencies/0/index", json!(-1)),
            ("/roots", json!([2])),
            ("/roots", json!([])),
            ("/units", json!([])),
            ("/units/0/dependencies", Value::Null),
            ("/units/0/features", Value::Null),
        ] {
            let mut altered = graph();
            *altered.pointer_mut(pointer).expect("existing field") = replacement;
            assert!(identity(&altered).is_err(), "{pointer}");
        }
        let mut duplicate = graph();
        let helper = duplicate["units"][1].clone();
        duplicate["units"].as_array_mut().expect("units").push(helper);
        assert!(identity(&duplicate).expect_err("ambiguous units").contains("duplicate"));
        for field in ["dependencies", "features"] {
            let mut missing = graph();
            missing["units"][0].as_object_mut().expect("unit").remove(field);
            assert!(identity(&missing).is_err(), "{field}");
        }
    }
}
