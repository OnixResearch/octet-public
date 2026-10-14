## Phase 1: Implementation

- [x] [serial] r[octet.runner_core_architecture.explicit_effects] Identify runner, standards, and hosted-core seams that currently mix deterministic decisions with filesystem, process, environment, clock, random, logging, metrics, network, or async-runtime effects.
- [x] [serial] r[octet.runner_core_architecture.explicit_effects] Extract selected deterministic decisions into pure functions that accept typed in-memory inputs and return decisions, diagnostics, events, or execution/write plans.
- [x] [serial] r[octet.runner_core_architecture.capability_traits] Introduce narrow capability traits or tokens only where passing concrete values is insufficient, and keep each capability focused on one effect family.
- [x] [serial] r[octet.runner_core_architecture.effect_tests] Add shell-boundary tests with explicit fake capabilities or captured command results while keeping pure-core tests free of side effects.
- [x] [serial] r[octet.runner_core_architecture.explicit_effects.non_claim] Update docs or comments to state the seam is a reviewable effect boundary, not a full Rust effect system or purity proof.

## Phase 2: Verification

- [x] [serial] r[octet.runner_core_architecture.explicit_effects.core_rejects_ambient] Verify migrated pure cores can be tested without filesystem, process, environment, clock, random, logging, metrics, network, or async runtime access.
- [x] [serial] r[octet.runner_core_architecture.capability_traits.narrow] Verify added traits or tokens are narrow, explicit, and not broad service objects that hide unrelated effects.
- [x] [serial] r[octet.runner_core_architecture.effect_tests.positive_negative] Verify positive shell cases enact returned plans and negative shell cases reject missing, malformed, timed-out, or failed effect results deterministically.
- [x] [serial] r[octet.runner_core_architecture.explicit_effects] Verify existing CLI output, exit codes, artifact formats, and evidence receipts remain stable unless explicitly changed.
- [x] [serial] Run focused runner/standards tests, relevant purity-effect lint checks where available, and Cairn validate/gates before closeout. [depends:cairn-gates]
