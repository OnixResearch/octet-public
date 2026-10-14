## Tasks

- [x] [serial] Define the Octet-to-standalone-Valence dependency manifest and migration map. r[octet.valence_project_extraction.phase5.standalone_dependency_manifest]
- [x] [serial] Add positive compatibility fixtures for Function Object v1, evidence-index, and evidence-codebase-index behavior. r[octet.valence_project_extraction.phase5.compatibility.positive]
- [x] [serial] Add negative fixtures for stale hosted files, wrong dependency source, missing fixture rows, and missing non-claims. r[octet.valence_project_extraction.phase5.compatibility.negative]
- [x] [serial] Implement pure cutover validation over dependency/source manifests plus a thin Octet shell for file discovery and receipt output. r[octet.valence_project_extraction.phase5.cutover_validation]
- [x] [serial] Move `cargo-octet` Valence call sites to the standalone dependency or an explicit compatibility shim. r[octet.valence_project_extraction.phase5.shell_consumes_standalone]
- [x] [serial] Document downstream migration guidance for Aspen and other Octet `valence-core` consumers. r[octet.valence_project_extraction.phase5.consumer_migration_docs]
- [x] [serial] Run focused Octet object/evidence gates, source-acquisition checks, and Cairn validation/gates. r[octet.valence_project_extraction.phase5.final_validation]
