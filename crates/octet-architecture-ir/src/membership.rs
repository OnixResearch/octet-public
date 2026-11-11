//! Complete, canonical observation membership; this is not source authenticity.

use std::collections::BTreeSet;

use crate::COMPILER_ARCHITECTURE_IR_SCHEMA_VERSION;
use crate::CompilerArchitectureIr;
use crate::CompilerArchitectureShard;
use crate::UnitFactMembership;

const ADJACENT_COUNT: usize = 2;

// r[impl octet.architecture.unit_membership]
pub(crate) fn memberships_from_shards(shards: &[CompilerArchitectureShard]) -> Vec<UnitFactMembership> {
    let mut rows = shards
        .iter()
        .map(|shard| UnitFactMembership {
            unit_id: shard.unit.unit_id.clone(),
            fact_ids: shard
                .facts
                .iter()
                .map(|fact| fact.fact_id().to_owned())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
            unknown_ids: shard
                .unknowns
                .iter()
                .map(|unknown| unknown.unknown_id.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.unit_id.cmp(&right.unit_id));
    rows
}

/// Check the normalized membership relation, independently of its enclosing digest.
/// This verifies consistency, not the authenticity of a compiler invocation.
#[must_use]
// r[impl octet.architecture.unit_membership]
pub fn verify_ir_unit_memberships(ir: &CompilerArchitectureIr) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if ir.schema_version != COMPILER_ARCHITECTURE_IR_SCHEMA_VERSION {
        diagnostics.push("compiler Architecture IR membership schema is unsupported".to_owned());
    }
    let units = ir.units.iter().map(|unit| unit.unit_id.as_str()).collect::<Vec<_>>();
    let facts = ir.facts.iter().map(crate::CompilerArchitectureFact::fact_id).collect::<Vec<_>>();
    let unknowns = ir.unknowns.iter().map(|unknown| unknown.unknown_id.as_str()).collect::<Vec<_>>();
    if units.is_empty() || !strictly_ordered(&units) || !strictly_ordered(&facts) || !strictly_ordered(&unknowns) {
        diagnostics.push("compiler Architecture IR membership subjects are empty or noncanonical".to_owned());
    }
    let row_units = ir.unit_memberships.iter().map(|row| row.unit_id.as_str()).collect::<Vec<_>>();
    if row_units != units {
        diagnostics.push("compiler Architecture IR memberships do not exactly cover canonical units".to_owned());
    }
    let expected_facts = facts.into_iter().collect::<BTreeSet<_>>();
    let expected_unknowns = unknowns.into_iter().collect::<BTreeSet<_>>();
    let mut observed_facts = BTreeSet::new();
    let mut observed_unknowns = BTreeSet::new();
    for row in &ir.unit_memberships {
        if !strictly_ordered(&row.fact_ids) || !strictly_ordered(&row.unknown_ids) {
            diagnostics.push("compiler Architecture IR membership IDs are not canonical".to_owned());
        }
        observed_facts.extend(row.fact_ids.iter().map(String::as_str));
        observed_unknowns.extend(row.unknown_ids.iter().map(String::as_str));
    }
    if observed_facts != expected_facts {
        diagnostics
            .push("compiler Architecture IR fact memberships contain orphan or unreferenced identities".to_owned());
    }
    if observed_unknowns != expected_unknowns {
        diagnostics
            .push("compiler Architecture IR unknown memberships contain orphan or unreferenced identities".to_owned());
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn strictly_ordered<T: Ord>(values: &[T]) -> bool {
    values.windows(ADJACENT_COUNT).all(|pair| pair[0] < pair[1])
}
