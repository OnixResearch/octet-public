## Why

The cross-repo smoke showed that `cargo octet check --function-addresses=observe` can compute function-address data but fail before writing `function-addresses.json` when unreviewed functions produce findings that violate the current finding-domain validator. Gradual adoption needs observe mode to be a safe discovery rail: operators should get the portable artifact and advisory debt without requiring a pre-existing baseline.

## What Changes

- Make observe-mode function-address findings valid for Octet's output domain.
- Ensure function-address artifacts are emitted even when observe mode reports missing markers or missing baseline entries.
- Preserve warn/deny behavior for stale or malformed reviewed baselines.
- Add positive smoke coverage for an unbaselined workspace and negative coverage for malformed/stale baseline failures.

## Impact

Octet becomes usable as the first step in the stack-wide function-address path without a bootstrap baseline workaround. Downstream Kamacite, Valence, Mantle, and Cairn smoke fixtures can consume the emitted artifact directly.
