## ADDED Requirements

### Requirement: Pedantic Clippy release-readiness coverage

r[octet.build_hygiene.clippy_pedantic] Octet's release-readiness Clippy seam SHOULD enable `clippy::pedantic` with warnings denied for owned workspace targets, and current pedantic findings SHOULD be fixed or justified narrowly rather than hidden behind broad suppression.

#### Scenario: Pedantic Clippy rail is clean

r[octet.build_hygiene.clippy_pedantic.clean]

- GIVEN the workspace is checked under the documented Nix devshell or flake check
- WHEN the Clippy release-readiness rail runs
- THEN owned Octet packages SHOULD pass with `-D warnings -W clippy::pedantic`.

#### Scenario: Pedantic finding is narrowly handled

r[octet.build_hygiene.clippy_pedantic.narrow]

- GIVEN a pedantic Clippy lint reports an owned-code issue
- WHEN the issue is addressed
- THEN the code SHOULD prefer a mechanical fix or focused refactor.
- AND any suppression SHOULD be narrow and tied to the smallest item where the lint conflicts with the existing contract.
