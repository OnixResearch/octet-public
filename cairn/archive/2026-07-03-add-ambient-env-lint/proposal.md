## Why

Environment reads and process arguments are hidden inputs. Pure functions should receive configuration, paths, and process context as explicit parameters, while shells own `std::env` and command-line capture.

## What Changes

- Define an `ambient_env` lint for direct environment access in production code.
- Cover `std::env` reads and mutations, including args/current-dir APIs, with test-context suppression and clear shell-boundary remediation.
- Require docs, metadata, and positive/negative fixtures before catalog inclusion.

## Impact

- **Files**: `src/purity/ambient_env.rs`, purity module registration, metadata catalog, lint reference, templates, and UI fixtures.
- **Testing**: fixtures for env var reads, args/current-dir reads, env mutation, injected config parameters, and shell-boundary allows.
