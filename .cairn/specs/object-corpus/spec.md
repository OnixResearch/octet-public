# Object Corpus Specification

## Purpose

Defines the `object-corpus` capability.

## Requirements

### Requirement: Reviewed object-corpus evidence is refreshed after source integration

r[octet.object_corpus.reviewed_refresh] The reviewed v2 object-corpus candidate
and reviewed manifests, reference receipts, caveat taxonomy, verifier anchors,
and reference counts MUST be refreshed together when the selected source bytes
change, including rail source integrations such as the measured Vermilion
canary. The refresh MUST preserve the 37-path/three-module selection, the four
reference artifacts, the Valence module/revision binding, and the explicit
mechanical-refresh non-claim.

#### Scenario: Refreshed source is admitted

- GIVEN the current selected source paths and the pinned Valence module
- WHEN the reviewed-source verifier runs
- THEN it MUST admit the refreshed manifests and references.

#### Scenario: Object-corpus gates replay the refreshed references

- GIVEN the refreshed internal and expanded references
- WHEN the fresh-build and caveat-taxonomy gates run
- THEN both MUST pass with the refreshed object counts and source buckets.

#### Scenario: Independent audit is not claimed

- GIVEN the mechanical refresh
- WHEN the approval scope is reviewed
- THEN it MUST state that the original independent audit was not repeated.
