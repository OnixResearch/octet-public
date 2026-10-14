## Context

Octet already supports local `cargo octet check`, Nix `nix run github:OnixResearch/octet#cargo-octet -- check`, local artifact directories, SARIF/sidecar output, and pre-commit integration. These are deployment features for regulated teams, but they are currently presented as setup mechanics rather than a controlled-deployment story.

The goal is documentation and checklist clarity, not a new hosted/offline mode.

## Approach

1. Add a controlled-deployment section to consumer-facing docs.
2. State the required claims precisely: Octet does not require a hosted service; it runs in local or consumer CI environments; artifacts are written to configured local paths; Nix and cargo paths can pin the toolchain.
3. Add a consumer-owned boundary caveat: Octet cannot control whether a user's CI later uploads logs or artifacts.
4. Provide a short checklist for regulated consumers.
5. Add documentation validation for required bullets and forbidden absolute exfiltration claims.

## Risks

- Absolute "source never leaves" language could be false in a user's CI environment. Mitigate with the consumer-owned boundary caveat.
- Deployment docs could duplicate quick-start instructions. Mitigate by linking to existing commands and adding only the governance interpretation.
- Checklist drift could confuse consumers. Mitigate by keeping it tied to current `cargo octet check`, Nix, pre-commit, and artifact-dir surfaces.

## Validation

- Positive: docs include local/CI execution, no required hosted service, pinned runner path, source-visible toolchain, and local artifact-directory bullets.
- Negative: docs do not contain absolute "source never leaves" claims without the consumer-owned boundary caveat.
- Cairn validation and gates pass for this change.
