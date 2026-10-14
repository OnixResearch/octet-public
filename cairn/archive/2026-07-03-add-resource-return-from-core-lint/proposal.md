## Why

Core functions that return concrete resources or capability handles blur the boundary between deterministic decisions and imperative effects. Pure cores should return values, plans, or port-independent commands instead of manufacturing files, sockets, commands, clients, or task handles.

## What Changes

- Define a `resource_return_from_core` lint for resource/capability return types in configured core scopes.
- Cover obvious handle types such as files, sockets, processes, commands, join handles, channels, and external clients where path matching is reliable.
- Require remediation toward data values, command descriptions, or shell-owned capabilities.

## Impact

- **Files**: `src/purity/resource_return_from_core.rs`, registration, metadata, lint reference, templates, and fixtures.
- **Testing**: positive return-type fixtures and guardrails for shell code, domain values, command descriptions, and injected ports.
