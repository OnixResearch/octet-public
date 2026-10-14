## ADDED Requirements

### Requirement: Controlled deployment claims

r[octet.controlled_deployment.claims] Octet documentation MUST describe the controlled-deployment posture: local or consumer-CI execution, no required hosted service, pin-capable runner path, source-visible toolchain, and local artifact-directory output.

#### Scenario: Controlled-deployment bullets are visible

r[octet.controlled_deployment.claims.visible]

- GIVEN a regulated consumer opens Octet setup or rollout documentation
- WHEN they inspect the controlled-deployment section
- THEN they MUST see that Octet can run locally or in the consumer's CI environment.
- AND they MUST see that Octet does not require a hosted service.
- AND they MUST see how to pin the runner path or revision.
- AND they MUST see that artifacts can be written to a configured local artifact directory.

#### Scenario: Exfiltration boundary is precise

r[octet.controlled_deployment.claims.no_absolute_exfiltration_claim]

- GIVEN documentation discusses source or artifact locality
- WHEN it states that Octet does not require uploading source or artifacts
- THEN it MUST also preserve the consumer-owned boundary caveat for downstream CI storage, logs, and integrations.
- AND an absolute claim that source never leaves the consumer environment MUST fail validation unless it is explicitly scoped to Octet's own required behavior.

### Requirement: Regulated-consumer checklist

r[octet.controlled_deployment.checklist] Octet documentation SHOULD provide a regulated-consumer checklist covering pinned Octet revision, checked-in configuration, deterministic artifact directory, and optional pre-commit gate.

#### Scenario: Checklist is actionable

r[octet.controlled_deployment.checklist.actionable]

- GIVEN a consumer wants a controlled Octet rollout
- WHEN they read the regulated-consumer checklist
- THEN it SHOULD name the pinned Octet revision or path, checked-in `dylint.toml`, configured artifact directory, and optional pre-commit gate.
- AND each checklist item SHOULD point to the command or template that implements it.
