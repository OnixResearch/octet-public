## Tasks

- [x] [serial] Add `cap-std` only to Octet runner/shell crates that own filesystem effects. r[octet.artifact_bundle_verification.cap_std_boundary.dependency]
- [x] [serial] Introduce typed capability roots for workspace, source, artifact, proof, and cache roots. r[octet.artifact_bundle_verification.cap_std_boundary.root_wrappers]
- [x] [serial] Convert artifact-dir, cache, proof, source-acquisition, and artifact-verifier opens to capability-relative operations. r[octet.artifact_bundle_verification.cap_std_boundary.conversion]
- [x] [serial] Add positive fixtures for valid artifact bundles, cache access, source reads, and proof receipt reads under declared roots. r[octet.artifact_bundle_verification.cap_std_boundary.tests.positive]
- [x] [serial] Add negative fixtures for traversal, absolute paths, symlink escapes, missing roots, and cache/output escape attempts. r[octet.artifact_bundle_verification.cap_std_boundary.tests.negative]
- [x] [serial] Document the local filesystem-authority boundary and checked-path non-claims. r[octet.artifact_bundle_verification.cap_std_boundary.docs]
- [x] [serial] Run focused runner/artifact-verifier tests plus Cairn validation and gates. r[octet.artifact_bundle_verification.cap_std_boundary.validation]
