use std::collections::BTreeMap;
use std::collections::BTreeSet;

use super::model::COMPARISON_PARENT_SCHEMA;
use super::model::COMPARISON_SCHEMA;
use super::model::ComparisonParent;
use super::model::ComparisonRow;
use super::model::ComparisonSummary;
use super::model::EffectCandidate;
use super::model::SemanticComparison;
use super::model::SemanticProjection;

const FUNCTION_ADDRESS_AUTHORITY: &str = "function-address";
const SOURCE_TOKEN_AUTHORITY: &str = "source-token";
const STATUS_AGREE: &str = "agree";
const STATUS_DISAGREE: &str = "disagree";
const STATUS_CHARON_ONLY: &str = "charon-only";
const STATUS_SOURCE_TOKEN_ONLY: &str = "source-token-only";
const STATUS_UNKNOWN: &str = "unknown";

pub(crate) fn compare(
    projection: &SemanticProjection,
    candidates: &[EffectCandidate],
    source_identity: &str,
    parents: &[ComparisonParent],
) -> Result<Option<SemanticComparison>, Vec<String>> {
    if parents.is_empty() {
        return Ok(None);
    }
    let mut diagnostics = validate_parents(parents);
    if !diagnostics.is_empty() {
        diagnostics.sort();
        diagnostics.dedup();
        return Err(diagnostics);
    }
    let charon_facts = projection_facts(projection, candidates);
    let mut rows = Vec::new();
    for parent in parents {
        rows.extend(compare_parent(&charon_facts, source_identity, parent));
    }
    rows.sort();
    rows.dedup();
    let summary = summarize(&rows);
    let mut parent_identities = parents.iter().map(|parent| parent.identity.clone()).collect::<Vec<_>>();
    parent_identities.sort();
    parent_identities.dedup();
    Ok(Some(SemanticComparison {
        schema_version: COMPARISON_SCHEMA.to_owned(),
        parent_identities,
        rows,
        summary,
        authority_merged: false,
    }))
}

fn validate_parents(parents: &[ComparisonParent]) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let mut identities = BTreeSet::new();
    for parent in parents {
        if parent.schema_version != COMPARISON_PARENT_SCHEMA {
            diagnostics.push(format!(
                "comparison parent `{}` has unsupported schema `{}`",
                parent.authority, parent.schema_version
            ));
        }
        if !matches!(parent.authority.as_str(), FUNCTION_ADDRESS_AUTHORITY | SOURCE_TOKEN_AUTHORITY) {
            diagnostics.push(format!("comparison parent has unsupported authority `{}`", parent.authority));
        }
        if !super::is_b3_identity(&parent.identity) || !super::is_b3_identity(&parent.source_identity) {
            diagnostics.push(format!("comparison parent `{}` has a malformed BLAKE3 identity", parent.authority));
        }
        if !identities.insert(parent.identity.clone()) {
            diagnostics.push(format!("duplicate comparison parent identity `{}`", parent.identity));
        }
        let mut facts = BTreeSet::new();
        for fact in &parent.facts {
            if fact.item.trim().is_empty() || fact.fact.trim().is_empty() || fact.value.trim().is_empty() {
                diagnostics.push(format!("comparison parent `{}` contains an empty fact field", parent.authority));
            }
            if !facts.insert((fact.item.clone(), fact.fact.clone())) {
                diagnostics.push(format!(
                    "comparison parent `{}` contains duplicate fact `{}` for `{}`",
                    parent.authority, fact.fact, fact.item
                ));
            }
        }
    }
    diagnostics
}

fn projection_facts(
    projection: &SemanticProjection,
    candidates: &[EffectCandidate],
) -> BTreeMap<(String, String), String> {
    let mut facts = BTreeMap::new();
    for item in &projection.items {
        facts.insert((item.qualified_name.clone(), "item-presence".to_owned()), "present".to_owned());
        facts.insert((item.qualified_name.clone(), "control-flow".to_owned()), item.control_flow.clone());
    }
    for edge in &projection.edges {
        facts.insert((edge.from.clone(), edge.kind.clone()), edge.to.clone());
    }
    for candidate in candidates {
        facts.insert((candidate.item.clone(), "effect-candidate".to_owned()), candidate.candidate.clone());
    }
    facts
}

fn compare_parent(
    charon_facts: &BTreeMap<(String, String), String>,
    source_identity: &str,
    parent: &ComparisonParent,
) -> Vec<ComparisonRow> {
    let parent_facts = parent
        .facts
        .iter()
        .map(|fact| ((fact.item.clone(), fact.fact.clone()), fact.value.clone()))
        .collect::<BTreeMap<_, _>>();
    if parent.source_identity != source_identity {
        return parent
            .facts
            .iter()
            .map(|fact| ComparisonRow {
                parent_authority: parent.authority.clone(),
                item: fact.item.clone(),
                fact: fact.fact.clone(),
                charon_value: charon_facts.get(&(fact.item.clone(), fact.fact.clone())).cloned(),
                parent_value: Some(fact.value.clone()),
                status: STATUS_UNKNOWN.to_owned(),
            })
            .collect();
    }
    let keys = charon_facts.keys().chain(parent_facts.keys()).cloned().collect::<BTreeSet<_>>();
    keys.into_iter()
        .map(|(item, fact)| {
            let charon_value = charon_facts.get(&(item.clone(), fact.clone())).cloned();
            let parent_value = parent_facts.get(&(item.clone(), fact.clone())).cloned();
            let status = match (&charon_value, &parent_value) {
                (Some(charon), Some(parent)) if charon == parent => STATUS_AGREE,
                (Some(_), Some(_)) => STATUS_DISAGREE,
                (Some(_), None) => STATUS_CHARON_ONLY,
                (None, Some(_)) => STATUS_SOURCE_TOKEN_ONLY,
                (None, None) => STATUS_UNKNOWN,
            };
            ComparisonRow {
                parent_authority: parent.authority.clone(),
                item,
                fact,
                charon_value,
                parent_value,
                status: status.to_owned(),
            }
        })
        .collect()
}

fn summarize(rows: &[ComparisonRow]) -> ComparisonSummary {
    let mut summary = ComparisonSummary::default();
    for row in rows {
        match row.status.as_str() {
            STATUS_AGREE => summary.agree += 1,
            STATUS_DISAGREE => summary.disagree += 1,
            STATUS_CHARON_ONLY => summary.charon_only += 1,
            STATUS_SOURCE_TOKEN_ONLY => summary.source_token_only += 1,
            _ => summary.unknown += 1,
        }
    }
    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charon_semantic::model::ComparisonFact;
    use crate::charon_semantic::model::SemanticItem;
    use crate::charon_semantic::model::SourceIdentity;
    use crate::charon_semantic::model::TargetMetadata;

    const SOURCE_IDENTITY: &str = "b3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const PARENT_IDENTITY: &str = "b3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    fn projection() -> SemanticProjection {
        SemanticProjection {
            schema_version: super::super::model::PROJECTION_SCHEMA.to_owned(),
            crate_name: "fixture".to_owned(),
            charon_version: "test".to_owned(),
            target: TargetMetadata {
                triple: "x86_64-unknown-linux-gnu".to_owned(),
                pointer_width_bytes: 8,
                endianness: "little".to_owned(),
            },
            sources: vec![SourceIdentity {
                path: "src/lib.rs".to_owned(),
                identity: SOURCE_IDENTITY.to_owned(),
            }],
            items: vec![SemanticItem {
                stable_id: "b3:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc".to_owned(),
                run_local_id: 0,
                qualified_name: "fixture::root".to_owned(),
                kind: "function".to_owned(),
                source_file: "src/lib.rs".to_owned(),
                begin_line: 1,
                end_line: 1,
                opacity: "Transparent".to_owned(),
                control_flow: "structured".to_owned(),
                statement_count: 1,
                unsupported: false,
            }],
            edges: Vec::new(),
            trait_rows: Vec::new(),
        }
    }

    #[test]
    fn preserves_agreement_disagreement_and_one_sided_facts() {
        let parent = ComparisonParent {
            schema_version: COMPARISON_PARENT_SCHEMA.to_owned(),
            authority: SOURCE_TOKEN_AUTHORITY.to_owned(),
            identity: PARENT_IDENTITY.to_owned(),
            source_identity: SOURCE_IDENTITY.to_owned(),
            facts: vec![
                ComparisonFact {
                    item: "fixture::root".to_owned(),
                    fact: "item-presence".to_owned(),
                    value: "present".to_owned(),
                },
                ComparisonFact {
                    item: "fixture::root".to_owned(),
                    fact: "control-flow".to_owned(),
                    value: "unstructured".to_owned(),
                },
                ComparisonFact {
                    item: "fixture::missing".to_owned(),
                    fact: "item-presence".to_owned(),
                    value: "present".to_owned(),
                },
            ],
        };
        let report = compare(&projection(), &[], SOURCE_IDENTITY, &[parent])
            .expect("comparison should validate")
            .expect("comparison should exist");
        assert_eq!(report.summary.agree, 1);
        assert_eq!(report.summary.disagree, 1);
        assert_eq!(report.summary.source_token_only, 1);
        assert!(!report.authority_merged);
    }

    #[test]
    fn stale_source_identity_remains_unknown() {
        let parent = ComparisonParent {
            schema_version: COMPARISON_PARENT_SCHEMA.to_owned(),
            authority: FUNCTION_ADDRESS_AUTHORITY.to_owned(),
            identity: PARENT_IDENTITY.to_owned(),
            source_identity: "b3:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd".to_owned(),
            facts: vec![ComparisonFact {
                item: "fixture::root".to_owned(),
                fact: "item-presence".to_owned(),
                value: "present".to_owned(),
            }],
        };
        let report = compare(&projection(), &[], SOURCE_IDENTITY, &[parent])
            .expect("comparison should validate")
            .expect("comparison should exist");
        assert_eq!(report.summary.unknown, 1);
    }

    #[test]
    fn rejects_duplicate_parent_facts() {
        let fact = ComparisonFact {
            item: "fixture::root".to_owned(),
            fact: "item-presence".to_owned(),
            value: "present".to_owned(),
        };
        let parent = ComparisonParent {
            schema_version: COMPARISON_PARENT_SCHEMA.to_owned(),
            authority: SOURCE_TOKEN_AUTHORITY.to_owned(),
            identity: PARENT_IDENTITY.to_owned(),
            source_identity: SOURCE_IDENTITY.to_owned(),
            facts: vec![fact.clone(), fact],
        };
        let diagnostics = compare(&projection(), &[], SOURCE_IDENTITY, &[parent]).expect_err("duplicates must fail");
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("duplicate fact")));
    }
}
