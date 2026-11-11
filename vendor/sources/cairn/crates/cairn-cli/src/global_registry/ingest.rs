use std::error::Error;

use cairn_core::{
    canonical_json_hash, RegistryAdapterMetadata, RegistryCandidateRecords, RegistryEdge,
    RegistryNode, RegistrySourceRecord, RegistryTagAssignment,
};
use serde_json::Value;

use super::policy::RegistryAdapterProfile;

const CAIRN_NON_CLAIM: &str =
    "Cairn registry ingestion preserves declared identity and linkage only; it does not prove lifecycle readiness or source correctness";
const VALENCE_NON_CLAIM: &str =
    "Valence vocabulary ownership is preserved; Cairn does not promote imported evidence roles into lifecycle authority";

// r[impl cairn.global_registry.adapters]
// r[impl cairn.global_registry.adapters.ingest]
// r[impl cairn.global_registry.graph.cairn]
// r[impl cairn.global_registry.graph.valence]
pub(super) fn ingest_registry_artifact(
    adapter: &RegistryAdapterProfile,
    input: &Value,
) -> Result<RegistryCandidateRecords, Box<dyn Error>> {
    let mut records = match adapter.id.as_str() {
        "cairn.canonical-records.v1" => serde_json::from_value(input.clone())?,
        "cairn.evidence-index.v1" => ingest_evidence_registry(input, adapter),
        "cairn.requirement-registry.v1" => ingest_requirement_registry(input, adapter),
        "cairn.change-graph.v1" => ingest_change_graph(input, adapter),
        "cairn.cross-repo-dependencies.v1" => ingest_cross_repo_dependencies(input, adapter),
        "valence.evidence-ir.v1" => ingest_valence(input, adapter),
        unknown => return Err(format!("unsupported registry ingest adapter: {unknown}").into()),
    };
    records.adapters.push(adapter_metadata(adapter));
    Ok(records)
}

fn ingest_evidence_registry(
    input: &Value,
    adapter: &RegistryAdapterProfile,
) -> RegistryCandidateRecords {
    let source = source_record(input, adapter, "cairn", CAIRN_NON_CLAIM);
    let entries = input
        .get("entries")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut nodes = Vec::new();
    let mut tags = Vec::new();
    for (position, entry) in entries.iter().enumerate() {
        let content_hash = entry
            .get("index_receipt_hash")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let stable_id = format!("evidence:{content_hash}");
        nodes.push(RegistryNode {
            stable_id: stable_id.clone(),
            kind: "evidence".to_string(),
            schema: "cairn.evidence-index.v1".to_string(),
            content_hash,
            payload_ref: format!("entries/{position}"),
            source_id: source.source_id.clone(),
            vocabulary_owner: "cairn".to_string(),
            non_claims: vec![CAIRN_NON_CLAIM.to_string()],
            record_hash: String::new(),
        });
        if let Some(labels) = entry.get("labels").and_then(Value::as_object) {
            for (key, value) in labels {
                if let Some(text) = value.as_str().filter(|text| !text.is_empty()) {
                    tags.push(RegistryTagAssignment {
                        assignment_id: format!("tag:{stable_id}:evidence:{key}:{text}"),
                        subject_id: stable_id.clone(),
                        subject_kind: "node".to_string(),
                        namespace: "evidence".to_string(),
                        key: key.clone(),
                        value: text.to_string(),
                        source_id: source.source_id.clone(),
                        non_claims: vec![CAIRN_NON_CLAIM.to_string()],
                        record_hash: String::new(),
                    });
                }
            }
        }
    }
    RegistryCandidateRecords {
        sources: vec![source],
        nodes,
        tags,
        ..RegistryCandidateRecords::default()
    }
}

fn ingest_requirement_registry(
    input: &Value,
    adapter: &RegistryAdapterProfile,
) -> RegistryCandidateRecords {
    let source = source_record(input, adapter, "cairn", CAIRN_NON_CLAIM);
    let entries = input
        .get("requirements")
        .or_else(|| input.get("entries"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let nodes = entries
        .iter()
        .enumerate()
        .map(|(position, entry)| {
            let stable_id = string_field(entry, &["stable_id", "requirement_id", "id"]);
            RegistryNode {
                stable_id,
                kind: "requirement".to_string(),
                schema: "cairn.requirement-registry-entry.v1".to_string(),
                content_hash: canonical_json_hash(entry),
                payload_ref: format!("requirements/{position}"),
                source_id: source.source_id.clone(),
                vocabulary_owner: "cairn".to_string(),
                non_claims: vec![CAIRN_NON_CLAIM.to_string()],
                record_hash: String::new(),
            }
        })
        .collect();
    RegistryCandidateRecords {
        sources: vec![source],
        nodes,
        ..RegistryCandidateRecords::default()
    }
}

fn ingest_change_graph(
    input: &Value,
    adapter: &RegistryAdapterProfile,
) -> RegistryCandidateRecords {
    let source = source_record(input, adapter, "cairn", CAIRN_NON_CLAIM);
    let node_values = input
        .get("nodes")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let nodes = node_values
        .iter()
        .enumerate()
        .map(|(position, value)| {
            let id = value
                .as_str()
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| string_field(value, &["stable_id", "change", "id"]));
            RegistryNode {
                stable_id: id,
                kind: "change".to_string(),
                schema: "cairn.change-graph-node.v1".to_string(),
                content_hash: canonical_json_hash(value),
                payload_ref: format!("nodes/{position}"),
                source_id: source.source_id.clone(),
                vocabulary_owner: "cairn".to_string(),
                non_claims: vec![CAIRN_NON_CLAIM.to_string()],
                record_hash: String::new(),
            }
        })
        .collect();
    let edge_values = input
        .get("edges")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let edges = edge_values
        .iter()
        .enumerate()
        .map(|(position, value)| {
            let from_id = string_field(value, &["from_id", "change", "from"]);
            let to_id = string_field(value, &["to_id", "dependency", "to"]);
            RegistryEdge {
                stable_id: value
                    .get("stable_id")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
                    .unwrap_or_else(|| format!("edge:change:{from_id}:{to_id}:{position}")),
                kind: "change_dependency".to_string(),
                from_id,
                to_id,
                vocabulary_owner: "cairn".to_string(),
                source_id: source.source_id.clone(),
                status: value
                    .get("status")
                    .and_then(Value::as_str)
                    .unwrap_or("declared")
                    .to_string(),
                non_claims: vec![CAIRN_NON_CLAIM.to_string()],
                record_hash: String::new(),
            }
        })
        .collect();
    RegistryCandidateRecords {
        sources: vec![source],
        nodes,
        edges,
        ..RegistryCandidateRecords::default()
    }
}

fn ingest_cross_repo_dependencies(
    input: &Value,
    adapter: &RegistryAdapterProfile,
) -> RegistryCandidateRecords {
    let source = source_record(input, adapter, "cairn", CAIRN_NON_CLAIM);
    let default_downstream = string_field(input, &["change", "downstream_id", "dependent_id"]);
    let mut rows = input
        .get("rows")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .cloned()
                .map(|row| (default_downstream.clone(), row))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if let Some(groups) = input
        .get("cross_repo_dependencies")
        .and_then(Value::as_array)
    {
        for group in groups {
            let downstream = string_field(group, &["change", "downstream_id"]);
            if let Some(group_rows) = group
                .get("report")
                .and_then(|report| report.get("rows"))
                .and_then(Value::as_array)
            {
                rows.extend(
                    group_rows
                        .iter()
                        .cloned()
                        .map(|row| (downstream.clone(), row)),
                );
            }
        }
    }
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    for (position, (downstream_hint, row)) in rows.iter().enumerate() {
        let dependency = row.get("dependency").unwrap_or(row);
        let upstream = format!(
            "change:{}:{}",
            string_field(dependency, &["repository_id", "repository"]),
            string_field(dependency, &["change_id", "change"])
        );
        let downstream = if downstream_hint.is_empty() {
            string_field(row, &["downstream_id", "dependent_id"])
        } else {
            downstream_hint.clone()
        };
        for stable_id in [&downstream, &upstream] {
            nodes.push(RegistryNode {
                stable_id: stable_id.clone(),
                kind: "change".to_string(),
                schema: "cairn.cross-repo-change.v1".to_string(),
                content_hash: canonical_json_hash(&Value::String(stable_id.clone())),
                payload_ref: format!("rows/{position}"),
                source_id: source.source_id.clone(),
                vocabulary_owner: "cairn".to_string(),
                non_claims: vec![CAIRN_NON_CLAIM.to_string()],
                record_hash: String::new(),
            });
        }
        edges.push(RegistryEdge {
            stable_id: format!("edge:cross-repo:{downstream}:{upstream}"),
            kind: "cross_repo_dependency".to_string(),
            from_id: downstream,
            to_id: upstream,
            vocabulary_owner: "cairn".to_string(),
            source_id: source.source_id.clone(),
            status: if row.get("satisfied").and_then(Value::as_bool) == Some(true) {
                "declared"
            } else {
                "blocked"
            }
            .to_string(),
            non_claims: vec![CAIRN_NON_CLAIM.to_string()],
            record_hash: String::new(),
        });
    }
    RegistryCandidateRecords {
        sources: vec![source],
        nodes,
        edges,
        ..RegistryCandidateRecords::default()
    }
}

fn ingest_valence(input: &Value, adapter: &RegistryAdapterProfile) -> RegistryCandidateRecords {
    let source = source_record(input, adapter, "valence", VALENCE_NON_CLAIM);
    let node_values = input
        .get("nodes")
        .or_else(|| input.get("records"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let nodes = node_values
        .iter()
        .enumerate()
        .map(|(position, value)| RegistryNode {
            stable_id: string_field(value, &["stable_id", "evidence_id", "id"]),
            kind: value
                .get("kind")
                .and_then(Value::as_str)
                .unwrap_or("evidence")
                .to_string(),
            schema: value
                .get("schema")
                .and_then(Value::as_str)
                .unwrap_or("valence.evidence-ir.v1")
                .to_string(),
            content_hash: value
                .get("content_hash")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            payload_ref: format!("nodes/{position}"),
            source_id: source.source_id.clone(),
            vocabulary_owner: "valence".to_string(),
            non_claims: vec![VALENCE_NON_CLAIM.to_string()],
            record_hash: String::new(),
        })
        .collect();
    let edge_values = input
        .get("edges")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let edges = edge_values
        .iter()
        .enumerate()
        .map(|(position, value)| RegistryEdge {
            stable_id: value
                .get("stable_id")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| format!("edge:valence:{position}")),
            kind: string_field(value, &["kind", "relation"]),
            from_id: string_field(value, &["from_id", "from"]),
            to_id: string_field(value, &["to_id", "to"]),
            vocabulary_owner: "valence".to_string(),
            source_id: source.source_id.clone(),
            status: "declared".to_string(),
            non_claims: vec![VALENCE_NON_CLAIM.to_string()],
            record_hash: String::new(),
        })
        .collect();
    RegistryCandidateRecords {
        sources: vec![source],
        nodes,
        edges,
        ..RegistryCandidateRecords::default()
    }
}

fn source_record(
    input: &Value,
    adapter: &RegistryAdapterProfile,
    vocabulary_owner: &str,
    non_claim: &str,
) -> RegistrySourceRecord {
    let content_hash = canonical_json_hash(input);
    RegistrySourceRecord {
        source_id: format!("source:{}:{content_hash}", adapter.id),
        family: adapter
            .record_families
            .first()
            .cloned()
            .unwrap_or_else(|| "unknown".to_string()),
        schema: adapter.schema.clone(),
        content_hash,
        vocabulary_owner: vocabulary_owner.to_string(),
        source_ref: format!("input:{}", adapter.id),
        non_claims: vec![non_claim.to_string()],
        record_hash: String::new(),
    }
}

fn adapter_metadata(adapter: &RegistryAdapterProfile) -> RegistryAdapterMetadata {
    RegistryAdapterMetadata {
        adapter_id: adapter.id.clone(),
        adapter_kind: adapter.kind.clone(),
        schema: adapter.schema.clone(),
        capabilities: adapter.capabilities.clone(),
        record_families: adapter.record_families.clone(),
    }
}

fn string_field(value: &Value, candidates: &[&str]) -> String {
    candidates
        .iter()
        .find_map(|field| value.get(field).and_then(Value::as_str))
        .unwrap_or_default()
        .to_string()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn profile(id: &str, family: &str) -> RegistryAdapterProfile {
        RegistryAdapterProfile {
            id: id.to_string(),
            kind: "ingest".to_string(),
            schema: "cairn.registry-adapter.v1".to_string(),
            capabilities: vec!["read".to_string()],
            record_families: vec![family.to_string()],
        }
    }

    #[test]
    // r[verify cairn.global_registry.adapters.ingest]
    // r[verify cairn.global_registry.graph.cairn]
    // r[verify cairn.global_registry.graph.valence]
    // r[verify cairn.global_registry.valence_boundary]
    // r[verify cairn.global_registry.valence_boundary.envelope]
    fn compiled_ingest_families_preserve_source_ownership() {
        let fixtures = [
            (
                profile("cairn.evidence-index.v1", "evidence-index"),
                json!({"entries": [{"index_receipt_hash": HASH_A, "labels": {"channel": "dev"}}]}),
                "cairn",
            ),
            (
                profile("cairn.requirement-registry.v1", "requirement-registry"),
                json!({"requirements": [{"id": "req:a"}]}),
                "cairn",
            ),
            (
                profile("cairn.change-graph.v1", "change-graph"),
                json!({"nodes": ["change:a", "change:b"], "edges": [{"from": "change:a", "to": "change:b"}]}),
                "cairn",
            ),
            (
                profile(
                    "cairn.cross-repo-dependencies.v1",
                    "cross-repo-dependencies",
                ),
                json!({"rows": [{"downstream_id": "change:local", "dependency": {"repository_id": "upstream", "change_id": "change:remote"}, "satisfied": false}]}),
                "cairn",
            ),
            (
                profile("valence.evidence-ir.v1", "valence-evidence-ir"),
                json!({"nodes": [{"id": "evidence:a", "kind": "evidence", "schema": "valence.evidence.v1", "content_hash": HASH_A}]}),
                "valence",
            ),
        ];
        for (adapter, input, expected_owner) in fixtures {
            let records = ingest_registry_artifact(&adapter, &input).unwrap();
            assert_eq!(records.sources[0].vocabulary_owner, expected_owner);
            assert!(!records.nodes.is_empty(), "adapter {}", adapter.id);
            assert_eq!(records.adapters[0].adapter_id, adapter.id);
            assert!(records
                .nodes
                .iter()
                .all(|node| node.vocabulary_owner == expected_owner));
        }
    }
}
