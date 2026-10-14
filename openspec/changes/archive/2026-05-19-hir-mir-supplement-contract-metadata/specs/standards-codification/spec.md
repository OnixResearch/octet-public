## ADDED Requirements

### Requirement: HIR/MIR supplement contract metadata

Octet MUST expose planned HIR/MIR supplemental evidence families in object corpus contract metadata as unsupported until deterministic extractor fixtures exist.

#### Scenario: Contract metadata names planned HIR/MIR supplements

- GIVEN `cargo octet object corpus contract --output-format json`
- WHEN the contract metadata is inspected
- THEN it lists HIR and MIR supplemental evidence families with schema IDs, schema versions, required future fields, and relationship to Function Object v1

#### Scenario: Planned supplements fail closed before extractor implementation

- GIVEN Octet has not implemented HIR or MIR extractors
- WHEN contract metadata describes HIR or MIR supplement status
- THEN each family reports `unsupported-planned` rather than supported or partial support
- AND each family records an unsupported reason and required deterministic fixture evidence before support can be claimed

#### Scenario: Human contract output preserves boundaries

- GIVEN an operator requests human object corpus contract output
- WHEN HIR/MIR supplement metadata is printed
- THEN the output says the families are supplemental, toolchain-scoped, and unsupported-planned
- AND it does not claim correctness, semantic equivalence, or cross-toolchain stability
