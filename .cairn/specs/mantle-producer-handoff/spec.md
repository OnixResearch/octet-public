# Mantle Producer Handoff Specification

## Purpose

Defines the `mantle-producer-handoff` capability.

## Requirements

### Requirement: Registered owner job preserves existing external proof authority

The producer MUST bind its existing consumer/provider profile identities, immutable source revision, execution commands, resource policy and output contract in a bounded Nickel-authored job. Its ordinary derivation MUST run the existing proof and artifact verifier and retain their original reports, envelope and payload bytes. It MUST reject external failure rather than interpreting process success as theorem acceptance. r[octet.mantle_producer_handoff.contract]

#### Scenario: Exact supported owner job

- GIVEN the immutable selected producer and complete declared tools/source inputs
- WHEN the ordinary job executes the existing external-fixture rail and artifact verifier
- THEN its original supported external receipt, valid artifact report, source/profile/tool identities and full payload tree are preserved
- AND no Mantle execution, signature trust or release authority is inferred.

#### Scenario: External rejection

- GIVEN an unsuccessful proof, invalid artifact report or unsupported consumer outcome
- WHEN the ordinary job observes the result
- THEN it fails closed without constructing an accepted external receipt from exit status.

### Requirement: Immutable graph acquisition is complete, bounded and byte-preserving

The exporter MUST require an immutable committed producer revision and evaluate its locked source. It MUST preserve the exact recursive graph bytes, selected root, flake metadata/lock and declared source objects under explicit size/member/byte bounds. It MUST admit legacy and explicit Nix v4 graph shapes separately and reject unknown/mixed formats, incomplete edges and dynamic outputs. It MUST NOT rewrite an original graph or replace its hash-domain identity. r[octet.mantle_producer_handoff.acquisition]

#### Scenario: Complete actual immutable export

- GIVEN an exact committed producer revision
- WHEN acquisition exports the selected ordinary job
- THEN the complete graph and source-object identities come from that immutable locked source
- AND no mutable-checkout job or fixture pin substitutes for the requested source.

#### Scenario: Unsupported or incomplete graph

- GIVEN an unknown/mixed graph version, absent dependency, dynamic output or exceeded bound
- WHEN the exporter admits the graph
- THEN acquisition rejects the input and does not silently normalize it into another accepted format.

### Requirement: Handoff observations keep owner roles and non-claims separate

Documentation and evidence MUST distinguish graph acquisition, actual owner execution, original cached verification and Mantle consumer acceptance. Octet MUST retain theorem/profile/tool admission; Mantle MUST retain build/store/cache/replay observations; Valence, Cairn and Trellis MUST retain their separate linkage, lifecycle and theorem-campaign authority. An exported graph, attached receipt, cached output or lifecycle gate MUST NOT grant correctness, trust, seed-free bootstrap or release authority. r[octet.mantle_producer_handoff.boundary]

#### Scenario: Cache or graph-only evidence

- GIVEN a successful graph export or reuse of an existing producer output
- WHEN its evidence is handed to Mantle
- THEN evidence identifies its exact source and route and preserves the original external verification bytes
- AND it does not claim fresh proof work, source-only tools, Mantle execution or release acceptance.
