//! Validated production evidence, never a filtered IR carrying the original IR identity.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use octet_architecture_ir::ArchitectureCollectionLimits;
use octet_architecture_ir::UnitFactMembership;
use octet_architecture_ir::UnknownCompilerFact;

use super::ArchitectureCargoGraph;
use super::CompilationUnit;
use super::CompilerArchitectureFact;
use super::CompilerArchitectureIr;
use super::CompilerArchitectureRole;
use super::ProjectArchitecturePolicy;
use super::SourceOrigin;
use super::SourceSpan;
use super::TestOnlyDisposition;
use super::fact_package;
use super::fact_span_id;
use super::path_in_scope;

/// Only this projection can enter production policy evaluation.
pub(super) struct ProductionView {
    pub units: Vec<CompilationUnit>,
    pub facts: Vec<CompilerArchitectureFact>,
    pub unknowns: Vec<UnknownCompilerFact>,
    spans: Vec<SourceSpan>,
    memberships: Vec<UnitFactMembership>,
    pub disposition: TestOnlyDisposition,
}

impl ProductionView {
    // r[impl octet.project_architecture_gate.test_ownership]
    pub fn new(ir: &CompilerArchitectureIr, limits: &ArchitectureCollectionLimits) -> Result<Self, Vec<String>> {
        let mut errors = octet_architecture_ir::verify_ir_unit_memberships(ir);
        errors.extend(octet_architecture_ir::verify_ir_unit_bindings(ir));
        errors.extend(octet_architecture_ir::verify_observation_set(&ir.facts, &ir.unknowns, &ir.spans, limits));
        if !errors.is_empty() {
            return Err(errors);
        }
        let test_units = ir
            .units
            .iter()
            .filter(|unit| unit.role == CompilerArchitectureRole::Test)
            .map(|unit| unit.unit_id.clone())
            .collect::<BTreeSet<_>>();
        let memberships = ir
            .unit_memberships
            .iter()
            .filter(|row| !test_units.contains(&row.unit_id))
            .cloned()
            .collect::<Vec<_>>();
        let fact_ids =
            memberships.iter().flat_map(|row| row.fact_ids.iter().map(String::as_str)).collect::<BTreeSet<_>>();
        let unknown_ids = memberships
            .iter()
            .flat_map(|row| row.unknown_ids.iter().map(String::as_str))
            .collect::<BTreeSet<_>>();
        let disposition = TestOnlyDisposition {
            unit_ids: test_units.iter().cloned().collect(),
            fact_ids: ir
                .facts
                .iter()
                .filter(|fact| !fact_ids.contains(fact.fact_id()))
                .map(|fact| fact.fact_id().to_owned())
                .collect(),
            unknown_ids: ir
                .unknowns
                .iter()
                .filter(|unknown| !unknown_ids.contains(unknown.unknown_id.as_str()))
                .map(|unknown| unknown.unknown_id.clone())
                .collect(),
        };
        let facts = ir.facts.iter().filter(|fact| fact_ids.contains(fact.fact_id())).cloned().collect::<Vec<_>>();
        let unknowns = ir
            .unknowns
            .iter()
            .filter(|unknown| unknown_ids.contains(unknown.unknown_id.as_str()))
            .cloned()
            .collect::<Vec<_>>();
        // Exclusion must not turn missing type/owner evidence into a skipped production check.
        let errors = octet_architecture_ir::verify_observation_set(&facts, &unknowns, &ir.spans, limits);
        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(Self {
            units: ir.units.iter().filter(|unit| !test_units.contains(&unit.unit_id)).cloned().collect(),
            facts,
            unknowns,
            spans: ir.spans.clone(),
            memberships,
            disposition,
        })
    }
}

/// No original IR, receipt, or coverage is available to the production evaluators.
pub(super) struct ProductionGateInput<'a> {
    pub policy: &'a ProjectArchitecturePolicy,
    /// Waiver-free policy identity; policy-sourced findings bind to it so waivers can name them.
    pub waiver_policy_blake3: &'a str,
    pub cargo_graph: &'a ArchitectureCargoGraph,
    /// A distinct type: this field has no normalized IR identity or schema.
    pub architecture_ir: &'a ProductionView,
}

pub(super) struct GateIndexes<'a> {
    ir: &'a ProductionView,
    units_by_package: BTreeMap<&'a str, &'a CompilationUnit>,
    fact_units: BTreeMap<&'a str, Vec<&'a CompilationUnit>>,
    unknown_units: BTreeMap<&'a str, Vec<&'a CompilationUnit>>,
    definition_units: BTreeMap<&'a str, &'a CompilationUnit>,
    spans: BTreeMap<&'a str, &'a SourceSpan>,
}

impl<'a> GateIndexes<'a> {
    pub fn new(ir: &'a ProductionView) -> Self {
        let mut units_by_package = BTreeMap::new();
        for unit in &ir.units {
            units_by_package.entry(unit.package_name.as_str()).or_insert(unit);
        }
        let units_by_id = ir.units.iter().map(|unit| (unit.unit_id.as_str(), unit)).collect::<BTreeMap<_, _>>();
        let mut fact_units: BTreeMap<&str, Vec<&CompilationUnit>> = BTreeMap::new();
        let mut unknown_units: BTreeMap<&str, Vec<&CompilationUnit>> = BTreeMap::new();
        for row in &ir.memberships {
            let unit = units_by_id[row.unit_id.as_str()];
            for id in &row.fact_ids {
                fact_units.entry(id).or_default().push(unit);
            }
            for id in &row.unknown_ids {
                unknown_units.entry(id).or_default().push(unit);
            }
        }
        let item_packages = ir
            .facts
            .iter()
            .filter_map(|fact| match fact {
                CompilerArchitectureFact::Item {
                    item_id, origin_crate, ..
                } => Some((item_id.as_str(), origin_crate.as_str())),
                _ => None,
            })
            .collect::<BTreeMap<_, _>>();
        let type_packages = ir
            .facts
            .iter()
            .filter_map(|fact| match fact {
                CompilerArchitectureFact::Type {
                    type_id, origin_crate, ..
                } => Some((type_id.as_str(), origin_crate.as_str())),
                _ => None,
            })
            .collect::<BTreeMap<_, _>>();
        let mut definition_units = BTreeMap::new();
        for fact in &ir.facts {
            let observed = &fact_units[fact.fact_id()];
            // Origin selects only among actual production observations; it never invents membership.
            // Implementation ownership is not ownership of its (possibly foreign) self type.
            let owner = if matches!(fact, CompilerArchitectureFact::Implementation { .. }) {
                None
            } else {
                fact_package(fact, &item_packages, &type_packages)
            };
            let unit = observed
                .iter()
                .find(|unit| Some(unit.package_name.as_str()) == owner)
                .or_else(|| observed.first())
                .expect("production fact has an observing unit");
            definition_units.insert(fact.fact_id(), *unit);
        }
        let spans = ir.spans.iter().map(|span| (span.span_id.as_str(), span)).collect();
        Self {
            ir,
            units_by_package,
            fact_units,
            unknown_units,
            definition_units,
            spans,
        }
    }

    pub fn unit_for_package(&self, package: &str) -> Option<&'a CompilationUnit> {
        self.units_by_package.get(package).copied()
    }

    pub fn units_for_fact(&self, fact_id: &str) -> &[&'a CompilationUnit] {
        self.fact_units.get(fact_id).map_or(&[], Vec::as_slice)
    }

    pub fn unit_for_fact(&self, fact_id: &str) -> Option<&'a CompilationUnit> {
        self.definition_units.get(fact_id).copied()
    }

    pub fn unit_for_unknown(&self, unknown_id: &str) -> Option<&'a CompilationUnit> {
        self.unknown_units.get(unknown_id).and_then(|units| units.first().copied())
    }

    pub fn source_path(&self, span_id: &str) -> Option<&'a str> {
        self.spans.get(span_id).and_then(|span| match &span.origin {
            SourceOrigin::Workspace { source_path } => Some(source_path.as_str()),
            SourceOrigin::ExternalGenerated { .. } => None,
        })
    }

    pub fn units_for_source_scope(&self, scope: &str) -> Vec<&'a CompilationUnit> {
        let mut units = BTreeMap::new();
        for fact in &self.ir.facts {
            if fact_span_id(fact)
                .and_then(|span| self.source_path(span))
                .is_some_and(|path| path_in_scope(path, scope))
            {
                for unit in self.units_for_fact(fact.fact_id()) {
                    units.insert(unit.unit_id.as_str(), *unit);
                }
            }
        }
        units.into_values().collect()
    }

    pub fn unit_has_configuration(&self, unit: &CompilationUnit, predicate: &str) -> bool {
        let expected = format!("{predicate}={}", unit.package_name);
        self.ir.facts.iter().any(|fact| {
            matches!(fact, CompilerArchitectureFact::Configuration { predicate: current, active: true, .. } if current == &expected)
                && self.units_for_fact(fact.fact_id()).iter().any(|observed| observed.unit_id == unit.unit_id)
        })
    }
}
