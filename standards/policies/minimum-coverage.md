# Tiger Style Hard-Rule Minimum Coverage Matrix

This matrix ties the OpenSpec `Hard Rule Families` minimum candidate list to the first-wave policy registry. It proves the registry has at least one policy ID for each required candidate before checker implementation begins.

| Spec family | Minimum candidate | Policy ID(s) | Planned task wave |
|---|---|---|---|
| Architecture | FCIS crate boundary | `tigerstyle.architecture.fcis-crate-boundary` | Architecture and Boundary Rules |
| Architecture | No external domain type leakage into core | `tigerstyle.architecture.external-domain-leakage` | Architecture and Boundary Rules |
| Architecture | Explicit conversion at boundaries | `tigerstyle.architecture.explicit-boundary-conversions` | Architecture and Boundary Rules |
| Architecture | Root crate thinness | `tigerstyle.architecture.root-crate-thinness` | Architecture and Boundary Rules |
| Architecture | Controlled public API surface | `tigerstyle.architecture.public-api-budget` | Architecture and Boundary Rules |
| DevTools | Single-command verification | `tigerstyle.devtools.single-command-verification` | Existing Toolchain Hardening; Final Verification |
| DevTools | Deny warnings | `tigerstyle.devtools.deny-warnings` | Existing Toolchain Hardening |
| DevTools | Formatting enforcement | `tigerstyle.devtools.formatting` | Existing Toolchain Hardening |
| DevTools | Documentation gates | `tigerstyle.devtools.documentation-gates` | Existing Toolchain Hardening |
| DevTools | Dependency audit gates | `tigerstyle.devtools.dependency-audit` | Existing Toolchain Hardening |
| DevTools | No unexplained lint suppressions | `tigerstyle.devtools.lint-suppressions` | Existing Toolchain Hardening |
| Embedded | Bounded channels and queues | `tigerstyle.embedded.bounded-queues` | Resource, Resilience, and Embedded Rules |
| Embedded | Timeout-backed blocking I/O | `tigerstyle.embedded.blocking-io-timeouts` | Resource, Resilience, and Embedded Rules |
| Embedded | Opt-in subsystem startup from hardware manifest | `tigerstyle.embedded.opt-in-subsystem-manifest` | Resource, Resilience, and Embedded Rules |
| Embedded | Safe serial APIs | `tigerstyle.embedded.safe-serial-api` | Resource, Resilience, and Embedded Rules |
| Embedded | Bounded task/thread topology | `tigerstyle.embedded.bounded-task-topology` | Resource, Resilience, and Embedded Rules |
| Resilience | Explicit typed errors | `tigerstyle.resilience.typed-errors` | Resource, Resilience, and Embedded Rules |
| Resilience | No silent defaulting on fallible paths | `tigerstyle.resilience.no-silent-defaults` | Resource, Resilience, and Embedded Rules |
| Resilience | Modeled failure modes | `tigerstyle.resilience.modeled-failure-modes` | Resource, Resilience, and Embedded Rules |
| Resilience | Fail-closed error handling | `tigerstyle.resilience.fail-closed-handling` | Resource, Resilience, and Embedded Rules |
| Orchestration | Declarative transition tables | `tigerstyle.orchestration.transition-tables` | Orchestration and Idiom Rules |
| Orchestration | Deny-by-default work acceptance | `tigerstyle.orchestration.deny-by-default-work-acceptance` | Orchestration and Idiom Rules |
| Orchestration | Invalid transitions leave state unchanged | `tigerstyle.orchestration.invalid-transition-preserves-state` | Orchestration and Idiom Rules |
| Orchestration | Priority ordering properties | `tigerstyle.orchestration.priority-ordering` | Orchestration and Idiom Rules |
| Idioms | One behavior / one type extraction candidates | `tigerstyle.idioms.one-behavior-one-type` | Orchestration and Idiom Rules |
| Idioms | Duplicate primitive-behavior detection | `tigerstyle.idioms.duplicate-primitive-behavior` | Orchestration and Idiom Rules |
| Idioms | Thin-forwarder verifier recognition | `tigerstyle.idioms.thin-forwarder-verifier` | Orchestration and Idiom Rules |
| Build What You Test | Builders for broad config setup | `tigerstyle.build-what-you-test.test-builders` | Test, Telemetry, and Audit Rules |
| Build What You Test | Visible preconditions | `tigerstyle.build-what-you-test.visible-preconditions` | Test, Telemetry, and Audit Rules |
| Build What You Test | No hidden external fixtures | `tigerstyle.build-what-you-test.no-hidden-external-fixtures` | Test, Telemetry, and Audit Rules |
| Build What You Test | Isolated process-global state | `tigerstyle.build-what-you-test.process-global-isolation` | Test, Telemetry, and Audit Rules |
| Metrics | Flat telemetry payloads | `tigerstyle.metrics.flat-telemetry` | Test, Telemetry, and Audit Rules |
| Metrics | Threshold-backed health checks | `tigerstyle.metrics.threshold-backed-health` | Test, Telemetry, and Audit Rules |
| Metrics | Prometheus text exposition | `tigerstyle.metrics.prometheus-text` | Test, Telemetry, and Audit Rules |
| Metrics | BLAKE3 hash-chain audit integrity | `tigerstyle.metrics.blake3-audit-chain` | Test, Telemetry, and Audit Rules |
| Verification | Positive and negative tests | `tigerstyle.verification.positive-negative-tests` | Test, Telemetry, and Audit Rules |
| Verification | FIRST properties where statically checkable | `tigerstyle.verification.first-properties` | Test, Telemetry, and Audit Rules |
| Verification | Exact error assertions | `tigerstyle.verification.exact-error-assertions` | Test, Telemetry, and Audit Rules |
| Verification | Target/hardware coverage declarations | `tigerstyle.verification.target-hardware-coverage` | Fitness and Safety Evidence |
| Verification | Safety-critical evidence prompts | `tigerstyle.verification.safety-critical-claims` | Fitness and Safety Evidence |

## External source and pattern-incorporation roadmap

These entries are not part of the original hard-rule minimum list. They capture future-rule staging for the Valgrind and Rust Design Patterns incorporation change.

| Source family | Candidate | Policy ID | First tier | Fixture gate before hard promotion |
|---|---|---|---|---|
| External sources | Classification before incorporation | `tigerstyle.external.source-classification` | hard registry/documentation rule | OpenSpec/package inventory checks |
| Dynamic evidence | Opt-in runtime/profile evidence profiles | `tigerstyle.devtools.dynamic-evidence-profiles` | advisory/human evidence rule | Tool-specific profile fixtures before any CI promotion |
| Rust idioms | Imported Rust idioms | `tigerstyle.idioms.imported-rust-idioms` | advisory guidance | Per-lint compliant and noncompliant fixtures |
| Functional APIs | Type-level state/protocol boundaries | `tigerstyle.idioms.functional-api-boundaries` | human-process-only | Human design review before any checker proposal |
