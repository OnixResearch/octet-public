#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, Ord, PartialOrd, serde::Serialize)]
pub(crate) struct FunctionAbi {
    pub(crate) direction: String,
    pub(crate) namespace: String,
    pub(crate) name: String,
    pub(crate) parameters: Vec<String>,
    pub(crate) results: Vec<String>,
}

// Resolve function indices through imports and definitions, never through export
// names or type-section order alone. This is a static ABI, not execution proof.
pub(crate) fn function_abis(bytes: &[u8]) -> Result<Vec<FunctionAbi>, String> {
    let mut types = Vec::new();
    let mut functions = Vec::new();
    let mut imports = Vec::new();
    let mut exports = Vec::new();
    for payload in wasmparser::Parser::new(0).parse_all(bytes) {
        match payload.map_err(|error| error.to_string())? {
            wasmparser::Payload::TypeSection(reader) => {
                for ty in reader.into_iter_err_on_gc_types() {
                    let ty = ty.map_err(|error| error.to_string())?;
                    types.push((values(ty.params()), values(ty.results())));
                }
            }
            wasmparser::Payload::ImportSection(reader) => {
                for import in reader.into_imports() {
                    let import = import.map_err(|error| error.to_string())?;
                    if let wasmparser::TypeRef::Func(index) = import.ty {
                        imports.push((import.module.to_owned(), import.name.to_owned(), index));
                        functions.push(index);
                    }
                }
            }
            wasmparser::Payload::FunctionSection(reader) => {
                for index in reader {
                    functions.push(index.map_err(|error| error.to_string())?);
                }
            }
            wasmparser::Payload::ExportSection(reader) => {
                for export in reader {
                    let export = export.map_err(|error| error.to_string())?;
                    if export.kind == wasmparser::ExternalKind::Func {
                        exports.push((export.name.to_owned(), export.index));
                    }
                }
            }
            _ => {}
        }
    }
    let mut abis = Vec::new();
    for (namespace, name, index) in imports {
        abis.push(resolve(&types, index, "import", namespace, name)?);
    }
    for (name, index) in exports {
        let ty = functions.get(index as usize).ok_or("missing exported function")?;
        abis.push(resolve(&types, *ty, "export", "module".to_owned(), name)?);
    }
    abis.sort();
    Ok(abis)
}

type Signature = (Vec<String>, Vec<String>);

fn values(values: &[wasmparser::ValType]) -> Vec<String> {
    values.iter().map(ToString::to_string).collect()
}

fn resolve(
    types: &[Signature],
    index: u32,
    direction: &str,
    namespace: String,
    name: String,
) -> Result<FunctionAbi, String> {
    let (parameters, results) = types.get(index as usize).ok_or("missing function type")?;
    Ok(FunctionAbi {
        direction: direction.to_owned(),
        namespace,
        name,
        parameters: parameters.clone(),
        results: results.clone(),
    })
}

pub(crate) fn host_abi_errors(bytes: &[u8], abis: &[FunctionAbi]) -> Vec<String> {
    let mut errors = Vec::new();
    for (direction, namespace, name, parameters) in [
        ("import", "molten", "observe", vec!["i32".to_owned()]),
        ("import", "molten", "pause", vec![]),
        ("export", "module", "run", vec![]),
    ] {
        if !abis.iter().any(|abi| {
            abi.direction == direction
                && abi.namespace == namespace
                && abi.name == name
                && abi.parameters == parameters
                && abi.results.is_empty()
        }) {
            errors.push(format!("pilot requires {direction} {namespace}::{name} {parameters:?} -> ()"));
        }
    }
    if !matches!(exported_total(bytes), Ok(true)) {
        errors.push("pilot requires exported mutable i32 global total".to_owned());
    }
    errors
}

fn exported_total(bytes: &[u8]) -> Result<bool, String> {
    let mut globals = Vec::new();
    let mut total = None;
    for payload in wasmparser::Parser::new(0).parse_all(bytes) {
        match payload.map_err(|error| error.to_string())? {
            // Imported globals precede defined globals in the global index space.
            wasmparser::Payload::ImportSection(reader) => {
                for import in reader.into_imports() {
                    if let wasmparser::TypeRef::Global(ty) = import.map_err(|error| error.to_string())?.ty {
                        globals.push(ty);
                    }
                }
            }
            wasmparser::Payload::GlobalSection(reader) => {
                for global in reader {
                    globals.push(global.map_err(|error| error.to_string())?.ty);
                }
            }
            wasmparser::Payload::ExportSection(reader) => {
                for export in reader {
                    let export = export.map_err(|error| error.to_string())?;
                    if export.name == "total" && export.kind == wasmparser::ExternalKind::Global {
                        total = Some(export.index);
                    }
                }
            }
            _ => {}
        }
    }
    Ok(total
        .and_then(|index| globals.get(index as usize))
        .is_some_and(|ty| ty.mutable && ty.content_type == wasmparser::ValType::I32))
}

pub(crate) fn molten_abi_errors(
    abis: &[FunctionAbi],
    memories: &[crate::wasm_artifact::WasmMemoryDeclaration],
) -> Vec<String> {
    let mut errors = Vec::new();
    for (name, parameters, results) in [
        ("molten_alloc", vec!["i32"], vec!["i32"]),
        ("molten_dealloc", vec!["i32", "i32"], vec![]),
        ("molten_run", vec!["i32", "i32"], vec!["i64"]),
    ] {
        if !abis.iter().any(|abi| {
            abi.direction == "export" && abi.name == name && abi.parameters == parameters && abi.results == results
        }) {
            errors.push(format!("molten.wasm.abi.v1 requires {name} {parameters:?} -> {results:?}"));
        }
    }
    for abi in abis.iter().filter(|abi| abi.direction == "import") {
        if abi.namespace != "molten:hostcall"
            || !matches!(abi.name.as_str(), "assert" | "send")
            || abi.parameters != ["i32", "i32"]
            || abi.results != ["i64"]
        {
            errors.push(format!("invalid declared molten hostcall ABI: {}::{}", abi.namespace, abi.name));
        }
    }
    if memories.len() != 1
        || !memories.iter().all(|memory| {
            memory.maximum_pages.is_some()
                && memory.initial_pages >= 1
                && !memory.memory64
                && !memory.shared
                && memory.page_size_log2 == 16
        })
    {
        errors.push("Molten pilot requires one bounded, unshared wasm32 memory of at least one page".to_owned());
    }
    errors
}

#[cfg(test)]
mod tests {
    #[test]
    fn exported_signature_resolves_imported_function_indices() {
        let bytes = wat::parse_str("(module (type (func (param i64))) (type (func (result i32))) (import \"host\" \"f\" (func (type 0))) (func (type 1) i32.const 7) (export \"run\" (func 1)))").unwrap();
        let facts = super::function_abis(&bytes).unwrap();
        let run = facts.iter().find(|fact| fact.direction == "export").unwrap();
        assert!(run.parameters.is_empty());
        assert_eq!(run.results, ["i32"]);
        let imported = facts.iter().find(|fact| fact.direction == "import").unwrap();
        assert_eq!(imported.parameters, ["i64"]);
    }

    #[test]
    fn exported_total_resolves_through_imported_globals() {
        let mutable_total = wat::parse_str("(module (import \"host\" \"g\" (global i32)) (global (mut i32) (i32.const 0)) (export \"total\" (global 1)))").unwrap();
        assert_eq!(super::exported_total(&mutable_total), Ok(true));
        let immutable_total = wat::parse_str("(module (import \"host\" \"g\" (global (mut i32))) (global i32 (i32.const 0)) (global (mut i32) (i32.const 0)) (export \"total\" (global 1)))").unwrap();
        assert_eq!(super::exported_total(&immutable_total), Ok(false));
    }

    #[test]
    fn hostless_profile_rejects_same_name_wrong_signature() {
        let registry =
            crate::wasm_artifact::parse_registry(include_str!("../../standards/wasm-artifact/generated/profiles.json"))
                .unwrap();
        for (source, denied) in [
            (include_str!("../../standards/wasm-artifact/spacewasm/fixtures/negative/wrong-signature.wat"), true),
            ("(module (func (export \"run\")))", false),
        ] {
            let bytes = wat::parse_str(source).unwrap();
            let report =
                crate::wasm_artifact::analyze_bytes(&bytes, &registry, "spacewasm-mvp", "b3:test".to_owned(), &[])
                    .unwrap();
            assert_eq!(report.details.findings.iter().any(|finding| finding.issue_code == "abi-mismatch"), denied);
        }
    }

    #[test]
    fn oversized_artifact_is_denied_before_any_fact_extraction() {
        let mut registry =
            crate::wasm_artifact::parse_registry(include_str!("../../standards/wasm-artifact/generated/profiles.json"))
                .unwrap();
        let profile = registry.profiles.iter_mut().find(|profile| profile.profile_id == "spacewasm-mvp").unwrap();
        profile.resource_limits.artifact_bytes = 8;
        let bytes = wat::parse_str("(module (func (export \"run\")))").unwrap();
        let report =
            crate::wasm_artifact::analyze_bytes(&bytes, &registry, "spacewasm-mvp", "b3:test".to_owned(), &[]).unwrap();
        assert!(report.details.findings.iter().any(|finding| finding.issue_code == "resource-limit"));
        assert_eq!(report.details.facts.artifact_kind, "unknown");
        assert!(report.details.facts.function_abis.is_empty());
    }

    #[test]
    fn pilot_requires_exact_import_signatures_and_mutable_total() {
        let registry =
            crate::wasm_artifact::parse_registry(include_str!("../../standards/wasm-artifact/generated/profiles.json"))
                .unwrap();
        let source = include_str!("../../standards/wasm-artifact/spacewasm/fixtures/positive/host-segments.wat");
        let variants = [
            (source.to_owned(), false),
            (
                source
                    .replace("(param i32)", "(param i64)")
                    .replace("call $observe", "i64.extend_i32_s call $observe"),
                true,
            ),
            (source.replace("(mut i32)", "i32").replace("global.set $total", "drop"), true),
            (source.replace("\"pause\"", "\"not-pause\""), true),
        ];
        for (source, denied) in variants {
            let bytes = wat::parse_str(source).unwrap();
            let report = crate::wasm_artifact::analyze_bytes(
                &bytes,
                &registry,
                "spacewasm-mvp-host-pilot",
                "b3:test".to_owned(),
                &[],
            )
            .unwrap();
            assert_eq!(
                report.details.findings.iter().any(|finding| finding.severity == "error"),
                denied,
                "{:?}",
                report.details.findings
            );
        }
    }

    #[test]
    fn pilot_admits_imports_by_signature_independent_of_core_type_order() {
        let registry =
            crate::wasm_artifact::parse_registry(include_str!("../../standards/wasm-artifact/generated/profiles.json"))
                .unwrap();
        let source = include_str!("../../standards/wasm-artifact/spacewasm/fixtures/positive/host-segments.wat");
        // Declaring `() -> ()` first moves pause to type 0 and observe to type 1.
        let reordered = source.replacen("(module", "(module (type (func))", 1);
        let wrong_signature = reordered
            .replace("(param i32)", "(param i64)")
            .replace("call $observe", "i64.extend_i32_s call $observe");
        for (source, denied) in [(reordered, false), (wrong_signature, true)] {
            let bytes = wat::parse_str(&source).unwrap();
            let report = crate::wasm_artifact::analyze_bytes(
                &bytes,
                &registry,
                "spacewasm-mvp-host-pilot",
                "b3:test".to_owned(),
                &[],
            )
            .unwrap();
            assert!(
                report.details.facts.imports.iter().any(|import| import.name == "pause" && import.kind == "Func(0)")
            );
            assert_eq!(
                report.details.findings.iter().any(|finding| finding.severity == "error"),
                denied,
                "{:?}",
                report.details.findings
            );
        }
    }

    #[test]
    fn molten_abi_requires_bounded_memory_and_exact_host_contract() {
        let registry =
            crate::wasm_artifact::parse_registry(include_str!("../../standards/wasm-artifact/generated/profiles.json"))
                .unwrap();
        let source = include_str!("../../standards/wasm-artifact/spacewasm/fixtures/positive/molten-abi.wat");
        for (source, denied) in [
            (source.to_owned(), false),
            (source.replace("1 2)", "1)"), true),
            (source.replace("\"assert\"", "\"clock\""), true),
            (source.replace("\"molten_alloc\"", "\"not_alloc\""), true),
            (source.replace("1 2)", "1 257)"), true),
        ] {
            let bytes = wat::parse_str(source).unwrap();
            let report = crate::wasm_artifact::analyze_bytes(
                &bytes,
                &registry,
                "spacewasm-molten-abi-pilot",
                "b3:test".to_owned(),
                &[],
            )
            .unwrap();
            assert_eq!(
                report.details.findings.iter().any(|finding| finding.severity == "error"),
                denied,
                "{:?}",
                report.details.findings
            );
        }
    }
}
