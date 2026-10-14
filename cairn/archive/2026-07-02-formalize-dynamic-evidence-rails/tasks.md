## Contract

- [x] [serial] Define dynamic evidence profile schema, allowed command shape, and not-default boundary. r[octet.dynamic_evidence.profile_registry]
- [x] [serial] Define dynamic evidence runner behavior and bounded log identity fields. r[octet.dynamic_evidence.runner]
- [x] [serial] Define evidence-tier boundaries and non-claims for Miri/Loom-style runs. r[octet.dynamic_evidence.tier_boundary]

## Implementation

- [x] [serial] Add pure profile validation and dynamic evidence receipt DTOs. r[octet.dynamic_evidence.profile_registry]
- [x] [serial] Add runner shell support for selected configured profiles only. r[octet.dynamic_evidence.runner]
- [x] [serial] Add Miri unsafe-boundary and Loom concurrency profile examples in docs/templates. r[octet.dynamic_evidence.profile_registry]
- [x] [serial] Wire dynamic evidence receipts into artifact verification without making them default artifacts. r[octet.dynamic_evidence.tier_boundary]

## Verification

- [x] [serial] Add positive configured-profile receipt fixture. r[octet.dynamic_evidence.runner]
- [x] [serial] Add negative unknown-profile, disallowed-command, missing-tool, and acceptance-signal mismatch fixtures. r[octet.dynamic_evidence.runner]
- [x] [serial] Add a starter-check regression proving dynamic tools are not required by default. r[octet.dynamic_evidence.profile_registry]
- [x] [serial] Run targeted runner tests, artifact verification tests, and Cairn validation/gates. r[octet.dynamic_evidence.tier_boundary]
