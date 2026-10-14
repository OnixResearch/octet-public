## Tasks

- [x] [serial] Define the Octet function-address handoff fields required for Kamacite Preserves receipt encoding. r[octet.function_address_export.preserves_handoff.contract]
- [x] [serial] Validate handoff completeness over in-memory function-address records. r[octet.function_address_export.preserves_handoff.core]
- [x] [serial] Keep JSON output documented as an operator/debug projection rather than the canonical stack receipt. r[octet.function_address_export.preserves_handoff.json_projection]
- [x] [serial] Add positive fixtures that Kamacite can encode without reparsing Rust or recomputing spans. r[octet.function_address_export.preserves_handoff.positive]
- [x] [serial] Add negative fixtures for missing spans, absolute paths, missing dependency kind, malformed BLAKE3, unsupported producer metadata, and overclaims. r[octet.function_address_export.preserves_handoff.negative]
- [x] [serial] Run Octet function-address tests and the Preserves-backed stack smoke. r[octet.function_address_export.preserves_handoff.validation]
