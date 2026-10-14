## Contract

- [x] [serial] Define the unsafe/trusted-boundary record fields and evidence-tier boundary. r[octet.unsafe_trust_boundary.record]
- [x] [serial] Define public unsafe API wrapper or exception expectations. r[octet.unsafe_trust_boundary.public_api]
- [x] [serial] Define FFI/trusted import boundary expectations. r[octet.unsafe_trust_boundary.ffi]
- [x] [serial] Define selected dynamic-evidence expectations for high-risk unsafe or FFI changes. r[octet.unsafe_trust_boundary.dynamic_evidence]
- [x] [serial] Define forbidden verified-unsafe and Salt-style `@trusted` claims. r[octet.unsafe_trust_boundary.boundaries]

## Implementation

- [x] [serial] Add the canonical unsafe/trusted-boundary record definition to the appropriate safety or Rust-idiom documentation. r[octet.unsafe_trust_boundary.record]
- [x] [parallel] Link or align `public_unsafe_api` lint documentation with the safe-wrapper and documented-exception paths. r[octet.unsafe_trust_boundary.public_api]
- [x] [parallel] Add FFI/trusted import review prompts covering handles, error mapping, ownership, and lifetime erasure. r[octet.unsafe_trust_boundary.ffi]
- [x] [parallel] Link the unsafe/FFI dynamic evidence profile from the canonical record and state when it is selected. r[octet.unsafe_trust_boundary.dynamic_evidence]
- [x] [serial] Add deterministic docs checks or fixtures for required fields, safe-wrapper example, documented-exception example, links, and forbidden overclaims. r[octet.unsafe_trust_boundary.boundaries]

## Verification

- [x] [serial] Run the positive unsafe-boundary field inventory check. r[octet.unsafe_trust_boundary.record]
- [x] [serial] Run the positive safe-wrapper and documented-exception example check. r[octet.unsafe_trust_boundary.public_api]
- [x] [serial] Run the positive FFI/trusted import prompt inventory check. r[octet.unsafe_trust_boundary.ffi]
- [x] [serial] Run the positive dynamic-evidence link check. r[octet.unsafe_trust_boundary.dynamic_evidence]
- [x] [serial] Run the negative forbidden-claim wording check. r[octet.unsafe_trust_boundary.boundaries]
- [x] [serial] Run `nix run .#cairn -- validate --root .` and Cairn gates for this change. r[octet.unsafe_trust_boundary.record]
