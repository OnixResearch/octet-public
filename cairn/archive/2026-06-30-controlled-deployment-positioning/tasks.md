## Contract

- [x] [serial] Define required controlled-deployment claims and caveats. r[octet.controlled_deployment.claims]
- [x] [serial] Define the regulated-consumer checklist items. r[octet.controlled_deployment.checklist]

## Implementation

- [x] [serial] Add a controlled-deployment section to `docs/consumer-guide.md` or README. r[octet.controlled_deployment.claims]
- [x] [serial] Add a regulated-consumer checklist covering pinned Octet revision, checked-in config, local artifact directory, and optional pre-commit gate. r[octet.controlled_deployment.checklist]
- [x] [serial] Cross-link downstream rollout guidance to the controlled-deployment section. r[octet.controlled_deployment.claims]

## Verification

- [x] [serial] Add or run a positive documentation check for local/CI execution, no required hosted service, pinned runner path, source-visible toolchain, and local artifact directory. r[octet.controlled_deployment.claims]
- [x] [serial] Add or run a negative documentation check rejecting absolute `source never leaves` claims without a consumer-owned boundary caveat. r[octet.controlled_deployment.claims]
- [x] [serial] Run `nix run path:/path/to/git/cairn#cairn -- validate --root .` and Cairn gates for this change. r[octet.controlled_deployment.checklist]
