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
const FACT_VALUE_SEPARATOR: &str = ", ";

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
            if !facts.insert((fact.item.clone(), fact.fact.clone(), fact.value.clone())) {
                diagnostics.push(format!(
                    "comparison parent `{}` contains duplicate fact `{}` value `{}` for `{}`",
                    parent.authority, fact.fact, fact.value, fact.item
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
        insert_fact(&mut facts, &item.qualified_name, "item-presence", "present");
        insert_fact(&mut facts, &item.qualified_name, "control-flow", &item.control_flow);
    }
    for edge in &projection.edges {
        insert_fact(&mut facts, &edge.from, &edge.kind, &edge.to);
    }
    for candidate in candidates {
        insert_fact(&mut facts, &candidate.item, "effect-candidate", &candidate.candidate);
    }
    render_fact_values(facts)
}

fn insert_fact(facts: &mut BTreeMap<(String, String), BTreeSet<String>>, item: &str, fact: &str, value: &str) {
    facts.entry((item.to_owned(), fact.to_owned())).or_default().insert(value.to_owned());
}

/// Multi-valued facts (several edges or effect candidates for one item) compare as their
/// sorted, de-duplicated value set so no value is dropped and input order cannot change a row.
fn render_fact_values(facts: BTreeMap<(String, String), BTreeSet<String>>) -> BTreeMap<(String, String), String> {
    facts
        .into_iter()
        .map(|(key, values)| (key, values.into_iter().collect::<Vec<_>>().join(FACT_VALUE_SEPARATOR)))
        .collect()
}

fn compare_parent(
    charon_facts: &BTreeMap<(String, String), String>,
    source_identity: &str,
    parent: &ComparisonParent,
) -> Vec<ComparisonRow> {
    let mut grouped_parent_facts = BTreeMap::new();
    for fact in &parent.facts {
        insert_fact(&mut grouped_parent_facts, &fact.item, &fact.fact, &fact.value);
    }
    let parent_facts = render_fact_values(grouped_parent_facts);
    if parent.source_identity != source_identity {
        return parent_facts
            .into_iter()
            .map(|((item, fact), parent_value)| ComparisonRow {
                parent_authority: parent.authority.clone(),
                charon_value: charon_facts.get(&(item.clone(), fact.clone())).cloned(),
                item,
                fact,
                parent_value: Some(parent_value),
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
    use crate::charon_semantic::model::SemanticEdge;
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

    fn source_token_parent(facts: &[(&str, &str, &str)]) -> ComparisonParent {
        ComparisonParent {
            schema_version: COMPARISON_PARENT_SCHEMA.to_owned(),
            authority: SOURCE_TOKEN_AUTHORITY.to_owned(),
            identity: PARENT_IDENTITY.to_owned(),
            source_identity: SOURCE_IDENTITY.to_owned(),
            facts: facts
                .iter()
                .map(|(item, fact, value)| ComparisonFact {
                    item: (*item).to_owned(),
                    fact: (*fact).to_owned(),
                    value: (*value).to_owned(),
                })
                .collect(),
        }
    }

    fn candidate(name: &str) -> EffectCandidate {
        EffectCandidate {
            item: "fixture::root".to_owned(),
            candidate: name.to_owned(),
            evidence: format!("explicit {name} construct"),
        }
    }

    fn row<'a>(report: &'a SemanticComparison, fact: &str) -> &'a ComparisonRow {
        report
            .rows
            .iter()
            .find(|row| row.item == "fixture::root" && row.fact == fact)
            .expect("comparison row should exist")
    }

    #[test]
    fn multi_edge_items_compare_every_edge_target() {
        let mut projection = projection();
        for target in ["fixture::b", "fixture::a"] {
            for kind in ["local-call", "item-dependency"] {
                projection.edges.push(SemanticEdge {
                    from: "fixture::root".to_owned(),
                    to: target.to_owned(),
                    kind: kind.to_owned(),
                });
            }
        }
        let parent = source_token_parent(&[
            ("fixture::root", "local-call", "fixture::a"),
            ("fixture::root", "local-call", "fixture::b"),
            ("fixture::root", "item-dependency", "fixture::a"),
        ]);
        let report = compare(&projection, &[], SOURCE_IDENTITY, &[parent])
            .expect("repeated parent facts with distinct values should validate")
            .expect("comparison should exist");
        let local_call = row(&report, "local-call");
        assert_eq!(local_call.charon_value.as_deref(), Some("fixture::a, fixture::b"));
        assert_eq!(local_call.parent_value.as_deref(), Some("fixture::a, fixture::b"));
        assert_eq!(local_call.status, STATUS_AGREE);
        let dependency = row(&report, "item-dependency");
        assert_eq!(dependency.charon_value.as_deref(), Some("fixture::a, fixture::b"));
        assert_eq!(dependency.parent_value.as_deref(), Some("fixture::a"));
        assert_eq!(dependency.status, STATUS_DISAGREE);
    }

    #[test]
    fn multi_candidate_items_are_order_independent() {
        let parent = source_token_parent(&[("fixture::root", "effect-candidate", "panic")]);
        let panic_last = compare(
            &projection(),
            &[candidate("drop"), candidate("panic")],
            SOURCE_IDENTITY,
            std::slice::from_ref(&parent),
        )
        .expect("comparison should validate")
        .expect("comparison should exist");
        let drop_last =
            compare(&projection(), &[candidate("panic"), candidate("panic"), candidate("drop")], SOURCE_IDENTITY, &[
                parent,
            ])
            .expect("comparison should validate")
            .expect("comparison should exist");
        assert_eq!(panic_last, drop_last);
        let effect = row(&panic_last, "effect-candidate");
        assert_eq!(effect.charon_value.as_deref(), Some("drop, panic"));
        assert_eq!(effect.parent_value.as_deref(), Some("panic"));
        assert_eq!(effect.status, STATUS_DISAGREE);
    }
}
