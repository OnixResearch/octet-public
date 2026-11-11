# Example Requirement Specification

## Purpose

Provides a small accepted specification for content-bound registry tests.

## Requirements

### Requirement: Current example identity
r[example.registry.current] The example system MUST preserve the current requirement body.

#### Scenario: Current body exports
- GIVEN the current accepted specification
- WHEN Cairn exports the content-bound registry
- THEN the registry MUST bind this requirement body.
