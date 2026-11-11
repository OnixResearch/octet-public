# Cross-tool benchmark experiment report

Source-linked report for profile `octet-cross-tool-architecture-v1`.

- Profile identity: `b3:85644c90d25fd73f231f6deb1c8351d6cfda65e4461fe5984d3b9be40af36476`
- Source revision: `github:OnixResearch/octet@2bdd581980eef7f6ec8363ffa20ab9f3c798c9ee`
- Rust toolchain: `nightly-2026-03-21`
- Evidence tier: `transcript_assertion`
- Machine artifact: `benchmark-experiment.json`
- Retained observations: `benchmark-experiment-observations.json`
- Declared before measurement: `true`

## Measurement provenance

Executed locally:

- `clippy`

Recorded fixtures:

- `cargo-pup`
- `octet`
- `repository-baseline-script`

Unavailable executables:

- `archunit-rust`
- `cargo-deny`
- `cargo-vet`
- `codeql`
- `semgrep`

## Tool and configuration pins

| Tool | Version | Source pin | Configuration | Availability |
| --- | --- | --- | --- | --- |
| `octet` | cargo-octet 0.1.0 | github:OnixResearch/octet@2bdd581980eef7f6ec8363ffa20ab9f3c798c9ee | standards/benchmark/corpus/octet-architecture.json (CheckedIn) | recorded |
| `cargo-pup` | cargo-pup 0.1.9 | crates.io:cargo_pup@0.1.9 | UpstreamArgs: cargo-pup architecture rules supplied through the reviewed probe argv | recorded |
| `archunit-rust` | archunit 0.0.2 | crates.io:archunit@0.0.2 | UpstreamArgs: ArchUnitRust test module rules declared by the benchmark harness | unavailable |
| `clippy` | clippy 0.1.96 | rustup:nightly-2026-03-21/clippy | UpstreamArgs: clippy default and restriction selection recorded in the probe argv | local |
| `cargo-deny` | cargo-deny 0.20.2 | crates.io:cargo-deny@0.20.2 | NotInstalled: cargo-deny advisories, bans, licenses, and sources policy | unavailable |
| `cargo-vet` | cargo-vet 0.10.3 | crates.io:cargo-vet@0.10.3 | NotInstalled: cargo-vet config.toml with reviewed audit imports | unavailable |
| `semgrep` | semgrep 1.184.0 | github:semgrep/semgrep@v1.184.0 | UpstreamArgs: reviewed token-pattern rules for the dependency-boundary and reexport tasks | unavailable |
| `codeql` | codeql 2.27.4 | github:github/codeql@v2.27.4 | UpstreamArgs: reviewed Rust query pack for dependency-boundary and reexport tasks | unavailable |
| `repository-baseline-script` | bash 5.3 | github:OnixResearch/octet@2bdd581980eef7f6ec8363ffa20ab9f3c798c9ee | standards/benchmark/assurance/repository-baseline.sh (CheckedIn) | recorded |

- `archunit-rust` unavailable: `archunit` is unavailable or unusable in the recorded fixture environment

- `cargo-deny` unavailable: `cargo-deny` is unavailable or unusable in the recorded fixture environment

- `cargo-vet` unavailable: `cargo-vet` is unavailable or unusable in the recorded fixture environment

- `semgrep` unavailable: `semgrep` is unavailable or unusable in the recorded fixture environment

- `codeql` unavailable: `codeql` is unavailable or unusable in the recorded fixture environment

## Applicability matrix

| Task | octet | cargo-pup | archunit-rust | clippy | cargo-deny | cargo-vet | semgrep | codeql | repository-baseline-script |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `dependency-boundary` | S | S | S | U | U | U | P | P | P |
| `reexports-aliases` | S | S | S | U | U | U | P | P | P |
| `macro-expansion` | S | U | U | U | U | U | U | U | U |
| `conditional-compilation` | S | U | U | U | U | U | U | U | U |
| `trait-method-references` | S | S | S | U | U | U | U | U | U |
| `workspace-targets` | S | S | S | U | U | U | U | U | U |
| `empty-selected-scope` | S | S | S | U | U | U | U | U | U |
| `stale-evidence` | S | U | U | U | U | U | U | U | P |
| `changed-policy` | S | U | U | U | U | U | U | U | P |
| `missing-execution` | S | U | U | U | U | U | U | U | U |
| `invalid-proof-source-linkage` | S | U | U | U | U | U | U | U | U |

S = supported, P = partial, U = unsupported. Unsupported cells are not clean evidence.

## Frozen corpus and adjudication

| Case | Variant | Expected | Source link | Source identity | Rationale | Reviewer |
| --- | --- | --- | --- | --- | --- | --- |
| `dependency-boundary-compliant` | Compliant | Clean | [standards/benchmark/corpus/crates/app/src/compliant.rs](../../standards/benchmark/corpus/crates/app/src/compliant.rs#dependency-boundary-compliant) | `b3:936954ff246914c7b9e67c7991da6f2296f8469b4cf1732afff523428a64eaa2` | The application shell calls only the core-owned Port trait and constructs no adapter type. | octet-architecture-reviewer |
| `dependency-boundary-violating` | Violating | Finding | [standards/benchmark/corpus/crates/app/src/violating.rs](../../standards/benchmark/corpus/crates/app/src/violating.rs#dependency-boundary-violating) | `b3:d88195b2eb597142b97159781bb2c80a6ff6b2be5134767574e0d5746971b168` | The application shell reaches core internals and constructs an outbound adapter, violating the dependency rule. | octet-architecture-reviewer |
| `reexport-alias-compliant` | Compliant | Clean | [standards/benchmark/corpus/crates/facade/src/compliant.rs](../../standards/benchmark/corpus/crates/facade/src/compliant.rs#reexport-alias-compliant) | `b3:296661147fbe3d482f64ee20a5cc8a7aa53e655c56add4c16ae625c95b32f21b` | Aliasing a core-owned port is a permitted public reexport. | octet-architecture-reviewer |
| `reexport-alias-violating` | Violating | Finding | [standards/benchmark/corpus/crates/facade/src/violating.rs](../../standards/benchmark/corpus/crates/facade/src/violating.rs#reexport-alias-violating) | `b3:600991fb24d064fca0c544a5cfe3bee47caf63e6837cef52af665da9e546e81e` | The facade reexports outbound-adapter internals and constructs the adapter directly. | octet-architecture-reviewer |
| `macro-expansion-compliant` | Compliant | Clean | [standards/benchmark/corpus/crates/facade/src/compliant_macro.rs](../../standards/benchmark/corpus/crates/facade/src/compliant_macro.rs#macro-expansion-compliant) | `b3:cbefd91bc520a61a9108033bcc56ef4b7f11acf07c0c391b6cde49fe3da4df77` | The macro expands only to the core-owned port method. | octet-architecture-reviewer |
| `macro-expansion-violating` | Violating | Finding | [standards/benchmark/corpus/crates/facade/src/violating_macro.rs](../../standards/benchmark/corpus/crates/facade/src/violating_macro.rs#macro-expansion-violating) | `b3:c93654c1708f4deb5e35fe5dd10d39b1bd1c468c555025cc53cdf6dcfbf4b828` | The macro hides an outbound-adapter construction from token-only callers but not from expansion-aware analysis. | octet-architecture-reviewer |
| `cfg-compliant` | Compliant | Clean | [standards/benchmark/corpus/crates/gated/src/compliant.rs](../../standards/benchmark/corpus/crates/gated/src/compliant.rs#cfg-compliant) | `b3:8eb69d100f53accb82744c34987a2daf5fae9de24027de201c99cea9a085f4c1` | The strict branch calls the port; the non-strict branch is inert and constructs nothing. | octet-architecture-reviewer |
| `cfg-violating` | Violating | Finding | [standards/benchmark/corpus/crates/gated/src/violating.rs](../../standards/benchmark/corpus/crates/gated/src/violating.rs#cfg-violating) | `b3:eb9e2867648f5341eb5dbaf01dd0be2f477763820a7116523b8d7544df1376e2` | With default features the non-strict branch constructs the adapter; the analyzer must state the selected configuration. | octet-architecture-reviewer |
| `cfg-disputed` | Disputed | Disputed | [standards/benchmark/corpus/crates/gated/src/disputed.rs](../../standards/benchmark/corpus/crates/gated/src/disputed.rs#cfg-disputed) | `b3:0fbd717de7cef443b4dd621da98cec312a9a92802ee93d9418aa1fa7fd430987` | Both feature branches contain the same direct adapter call; reviewers dispute whether this is one violation or two configuration-specific violations. | benchmark-oracle-reviewer |
| `trait-method-compliant` | Compliant | Clean | [standards/benchmark/corpus/crates/traits/src/compliant.rs](../../standards/benchmark/corpus/crates/traits/src/compliant.rs#trait-method-compliant) | `b3:5c45d0b972350425a9bdb8c1650591b9d67c00381969fda8539f0b0c552267ad` | The call is expressed through the trait method on a dyn Trait receiver. | octet-architecture-reviewer |
| `trait-method-violating` | Violating | Finding | [standards/benchmark/corpus/crates/traits/src/violating.rs](../../standards/benchmark/corpus/crates/traits/src/violating.rs#trait-method-violating) | `b3:fd83c857ce8080bae2af30ade9415a2c76148ac5de8d28098d68299b3992a89d` | The call resolves to an inherent method on the concrete outbound adapter. | octet-architecture-reviewer |
| `workspace-target-compliant` | Compliant | Clean | [standards/benchmark/corpus/crates/bin_target/src/main.rs](../../standards/benchmark/corpus/crates/bin_target/src/main.rs#workspace-target-compliant) | `b3:34494cd7b1307108e64ae8df4281bea508920f4b21885cba80fc9213bbd7fccb` | The composition-root binary wires an explicit port and constructs no adapter. | octet-architecture-reviewer |
| `workspace-target-violating` | Violating | Finding | [standards/benchmark/corpus/crates/bin_target/src/bin/violating.rs](../../standards/benchmark/corpus/crates/bin_target/src/bin/violating.rs#workspace-target-violating) | `b3:9d48b18508c68f34adcc0c7ee73f7847d488ebeb343635241bada62437871541` | A separate workspace binary target constructs the outbound adapter outside the composition-root port path. | octet-architecture-reviewer |
| `empty-selected-scope` | EmptyScope | EmptyScope | [standards/benchmark/corpus/crates/empty_scope/selected](../../standards/benchmark/corpus/crates/empty_scope/selected#empty-selected-scope) | `b3:af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262` | The selected child scope contains no Rust files; zero findings are not clean evidence of correctness. | benchmark-oracle-reviewer |
| `stale-evidence` | Violating | Finding | [standards/benchmark/assurance/stale-evidence.json](../../standards/benchmark/assurance/stale-evidence.json#stale-evidence) | `b3:99a97796329a551fb7d4a5095be75ddbafc96ea18c637d47bf3ee1127204eeca` | The retained evidence identity differs from the identity the claim requires. | assurance-reviewer |
| `valid-evidence-control` | Control | Clean | [standards/benchmark/assurance/valid-evidence-control.json](../../standards/benchmark/assurance/valid-evidence-control.json#valid-evidence-control) | `b3:abfc4bd720f58d7308eee25aa89cabdc883388e7ed27b947e89be4b59016aeda` | All retained identities agree, so a conforming assurance check accepts the claim. | assurance-reviewer |
| `changed-policy` | Violating | Finding | [standards/benchmark/assurance/changed-policy.json](../../standards/benchmark/assurance/changed-policy.json#changed-policy) | `b3:1d79832f17d114669e2e58f7817bcbc486ae230924ee2271cd221e8258789c86` | The recorded policy identity differs from the currently checked policy identity. | assurance-reviewer |
| `missing-execution` | Violating | Finding | [standards/benchmark/assurance/missing-execution.json](../../standards/benchmark/assurance/missing-execution.json#missing-execution) | `b3:aafd86b34ed7a4858f4da9ae1acb8e663c809d03b49eb48ecffcda8b8a1ed8f2` | The execution status is not_run, so no observed run supports the claim. | assurance-reviewer |
| `invalid-proof-source-linkage` | Violating | Finding | [standards/benchmark/assurance/invalid-proof-linkage.json](../../standards/benchmark/assurance/invalid-proof-linkage.json#invalid-proof-source-linkage) | `b3:9114f755b70b1b88c4ef4ba931b42f4c4a297b246528172b7cdd7447d2596402` | The proof source identity does not match the identity the linkage claims. | assurance-reviewer |

## Architecture detection results

| Tool | Task | Applicability | Availability | Comparable | TP | FP | FN | TN | Unsupported | Inconclusive | Incomparable | Precision bp | Recall bp |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `octet` | `dependency-boundary` | Supported | recorded | 2 | 1 | 0 | 0 | 1 | 0 | 0 | 0 | 10000 | 10000 |
| `octet` | `reexports-aliases` | Supported | recorded | 2 | 1 | 0 | 0 | 1 | 0 | 0 | 0 | 10000 | 10000 |
| `octet` | `macro-expansion` | Supported | recorded | 2 | 0 | 0 | 1 | 1 | 0 | 0 | 0 | n/a | 0 |
| `octet` | `conditional-compilation` | Supported | recorded | 1 | 0 | 1 | 0 | 0 | 0 | 1 | 0 | 0 | n/a |
| `octet` | `trait-method-references` | Supported | recorded | 2 | 1 | 0 | 0 | 1 | 0 | 0 | 0 | 10000 | 10000 |
| `octet` | `workspace-targets` | Supported | recorded | 2 | 1 | 0 | 0 | 1 | 0 | 0 | 0 | 10000 | 10000 |
| `octet` | `empty-selected-scope` | Supported | recorded | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | n/a | n/a |
| `cargo-pup` | `dependency-boundary` | Supported | recorded | 2 | 1 | 0 | 0 | 1 | 0 | 0 | 0 | 10000 | 10000 |
| `cargo-pup` | `reexports-aliases` | Supported | recorded | 2 | 1 | 0 | 0 | 1 | 0 | 0 | 0 | 10000 | 10000 |
| `cargo-pup` | `macro-expansion` | Unsupported | recorded | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `cargo-pup` | `conditional-compilation` | Unsupported | recorded | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 0 | n/a | n/a |
| `cargo-pup` | `trait-method-references` | Supported | recorded | 2 | 1 | 0 | 0 | 1 | 0 | 0 | 0 | 10000 | 10000 |
| `cargo-pup` | `workspace-targets` | Supported | recorded | 2 | 1 | 0 | 0 | 1 | 0 | 0 | 0 | 10000 | 10000 |
| `cargo-pup` | `empty-selected-scope` | Supported | recorded | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | n/a | n/a |
| `archunit-rust` | `dependency-boundary` | Supported | unavailable | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `archunit-rust` | `reexports-aliases` | Supported | unavailable | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `archunit-rust` | `macro-expansion` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `archunit-rust` | `conditional-compilation` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 0 | n/a | n/a |
| `archunit-rust` | `trait-method-references` | Supported | unavailable | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `archunit-rust` | `workspace-targets` | Supported | unavailable | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `archunit-rust` | `empty-selected-scope` | Supported | unavailable | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | n/a | n/a |
| `clippy` | `dependency-boundary` | Unsupported | local | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `clippy` | `reexports-aliases` | Unsupported | local | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `clippy` | `macro-expansion` | Unsupported | local | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `clippy` | `conditional-compilation` | Unsupported | local | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 0 | n/a | n/a |
| `clippy` | `trait-method-references` | Unsupported | local | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `clippy` | `workspace-targets` | Unsupported | local | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `clippy` | `empty-selected-scope` | Unsupported | local | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | n/a | n/a |
| `cargo-deny` | `dependency-boundary` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `cargo-deny` | `reexports-aliases` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `cargo-deny` | `macro-expansion` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `cargo-deny` | `conditional-compilation` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 0 | n/a | n/a |
| `cargo-deny` | `trait-method-references` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `cargo-deny` | `workspace-targets` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `cargo-deny` | `empty-selected-scope` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | n/a | n/a |
| `cargo-vet` | `dependency-boundary` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `cargo-vet` | `reexports-aliases` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `cargo-vet` | `macro-expansion` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `cargo-vet` | `conditional-compilation` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 0 | n/a | n/a |
| `cargo-vet` | `trait-method-references` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `cargo-vet` | `workspace-targets` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `cargo-vet` | `empty-selected-scope` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | n/a | n/a |
| `semgrep` | `dependency-boundary` | Partial | unavailable | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `semgrep` | `reexports-aliases` | Partial | unavailable | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `semgrep` | `macro-expansion` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `semgrep` | `conditional-compilation` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 0 | n/a | n/a |
| `semgrep` | `trait-method-references` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `semgrep` | `workspace-targets` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `semgrep` | `empty-selected-scope` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | n/a | n/a |
| `codeql` | `dependency-boundary` | Partial | unavailable | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `codeql` | `reexports-aliases` | Partial | unavailable | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `codeql` | `macro-expansion` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `codeql` | `conditional-compilation` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 0 | n/a | n/a |
| `codeql` | `trait-method-references` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `codeql` | `workspace-targets` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `codeql` | `empty-selected-scope` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | n/a | n/a |
| `repository-baseline-script` | `dependency-boundary` | Partial | recorded | 2 | 1 | 0 | 0 | 1 | 0 | 0 | 0 | 10000 | 10000 |
| `repository-baseline-script` | `reexports-aliases` | Partial | recorded | 2 | 1 | 0 | 0 | 1 | 0 | 0 | 0 | 10000 | 10000 |
| `repository-baseline-script` | `macro-expansion` | Unsupported | recorded | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `repository-baseline-script` | `conditional-compilation` | Unsupported | recorded | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 0 | n/a | n/a |
| `repository-baseline-script` | `trait-method-references` | Unsupported | recorded | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `repository-baseline-script` | `workspace-targets` | Unsupported | recorded | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `repository-baseline-script` | `empty-selected-scope` | Unsupported | recorded | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | n/a | n/a |

Precision and recall use only the declared comparable subset.

### Unsupported applicability cells

- `archunit-rust`: 2 unsupported task cells
- `cargo-deny`: 7 unsupported task cells
- `cargo-pup`: 2 unsupported task cells
- `cargo-vet`: 7 unsupported task cells
- `clippy`: 7 unsupported task cells
- `codeql`: 5 unsupported task cells
- `repository-baseline-script`: 5 unsupported task cells
- `semgrep`: 5 unsupported task cells

## Assurance results

| Tool | Task | Applicability | Availability | Comparable | TP | FP | FN | TN | Unsupported | Inconclusive | Incomparable | Precision bp | Recall bp |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `octet` | `stale-evidence` | Supported | recorded | 2 | 1 | 0 | 0 | 1 | 0 | 0 | 0 | 10000 | 10000 |
| `octet` | `changed-policy` | Supported | recorded | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 10000 | 10000 |
| `octet` | `missing-execution` | Supported | recorded | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 10000 | 10000 |
| `octet` | `invalid-proof-source-linkage` | Supported | recorded | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 10000 | 10000 |
| `cargo-pup` | `stale-evidence` | Unsupported | recorded | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `cargo-pup` | `changed-policy` | Unsupported | recorded | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | n/a | n/a |
| `cargo-pup` | `missing-execution` | Unsupported | recorded | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | n/a | n/a |
| `cargo-pup` | `invalid-proof-source-linkage` | Unsupported | recorded | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | n/a | n/a |
| `archunit-rust` | `stale-evidence` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `archunit-rust` | `changed-policy` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | n/a | n/a |
| `archunit-rust` | `missing-execution` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | n/a | n/a |
| `archunit-rust` | `invalid-proof-source-linkage` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | n/a | n/a |
| `clippy` | `stale-evidence` | Unsupported | local | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `clippy` | `changed-policy` | Unsupported | local | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | n/a | n/a |
| `clippy` | `missing-execution` | Unsupported | local | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | n/a | n/a |
| `clippy` | `invalid-proof-source-linkage` | Unsupported | local | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | n/a | n/a |
| `cargo-deny` | `stale-evidence` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `cargo-deny` | `changed-policy` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | n/a | n/a |
| `cargo-deny` | `missing-execution` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | n/a | n/a |
| `cargo-deny` | `invalid-proof-source-linkage` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | n/a | n/a |
| `cargo-vet` | `stale-evidence` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `cargo-vet` | `changed-policy` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | n/a | n/a |
| `cargo-vet` | `missing-execution` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | n/a | n/a |
| `cargo-vet` | `invalid-proof-source-linkage` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | n/a | n/a |
| `semgrep` | `stale-evidence` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `semgrep` | `changed-policy` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | n/a | n/a |
| `semgrep` | `missing-execution` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | n/a | n/a |
| `semgrep` | `invalid-proof-source-linkage` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | n/a | n/a |
| `codeql` | `stale-evidence` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | n/a | n/a |
| `codeql` | `changed-policy` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | n/a | n/a |
| `codeql` | `missing-execution` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | n/a | n/a |
| `codeql` | `invalid-proof-source-linkage` | Unsupported | unavailable | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | n/a | n/a |
| `repository-baseline-script` | `stale-evidence` | Partial | recorded | 2 | 1 | 0 | 0 | 1 | 0 | 0 | 0 | 10000 | 10000 |
| `repository-baseline-script` | `changed-policy` | Partial | recorded | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 10000 | 10000 |
| `repository-baseline-script` | `missing-execution` | Unsupported | recorded | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | n/a | n/a |
| `repository-baseline-script` | `invalid-proof-source-linkage` | Unsupported | recorded | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | n/a | n/a |

Precision and recall use only the declared comparable subset.

### Unsupported applicability cells

- `archunit-rust`: 4 unsupported task cells
- `cargo-deny`: 4 unsupported task cells
- `cargo-pup`: 4 unsupported task cells
- `cargo-vet`: 4 unsupported task cells
- `clippy`: 4 unsupported task cells
- `codeql`: 4 unsupported task cells
- `repository-baseline-script`: 2 unsupported task cells
- `semgrep`: 4 unsupported task cells

## Retained failures, timeouts, and unparsed runs

- `octet` / `conditional-compilation` / `cfg-violating`: TimedOut (recorded fixture timeout)
- `archunit-rust` / `dependency-boundary` / `dependency-boundary-compliant`: Unavailable (`archunit` is unavailable or unusable in the recorded fixture environment)
- `archunit-rust` / `dependency-boundary` / `dependency-boundary-violating`: Unavailable (`archunit` is unavailable or unusable in the recorded fixture environment)
- `archunit-rust` / `reexports-aliases` / `reexport-alias-compliant`: Unavailable (`archunit` is unavailable or unusable in the recorded fixture environment)
- `archunit-rust` / `reexports-aliases` / `reexport-alias-violating`: Unavailable (`archunit` is unavailable or unusable in the recorded fixture environment)
- `archunit-rust` / `trait-method-references` / `trait-method-compliant`: Unavailable (`archunit` is unavailable or unusable in the recorded fixture environment)
- `archunit-rust` / `trait-method-references` / `trait-method-violating`: Unavailable (`archunit` is unavailable or unusable in the recorded fixture environment)
- `archunit-rust` / `workspace-targets` / `workspace-target-compliant`: Unavailable (`archunit` is unavailable or unusable in the recorded fixture environment)
- `archunit-rust` / `workspace-targets` / `workspace-target-violating`: Unavailable (`archunit` is unavailable or unusable in the recorded fixture environment)
- `archunit-rust` / `empty-selected-scope` / `empty-selected-scope`: Unavailable (`archunit` is unavailable or unusable in the recorded fixture environment)
- `semgrep` / `dependency-boundary` / `dependency-boundary-compliant`: Unavailable (`semgrep` is unavailable or unusable in the recorded fixture environment)
- `semgrep` / `dependency-boundary` / `dependency-boundary-violating`: Unavailable (`semgrep` is unavailable or unusable in the recorded fixture environment)
- `semgrep` / `reexports-aliases` / `reexport-alias-compliant`: Unavailable (`semgrep` is unavailable or unusable in the recorded fixture environment)
- `semgrep` / `reexports-aliases` / `reexport-alias-violating`: Unavailable (`semgrep` is unavailable or unusable in the recorded fixture environment)
- `codeql` / `dependency-boundary` / `dependency-boundary-compliant`: Unavailable (`codeql` is unavailable or unusable in the recorded fixture environment)
- `codeql` / `dependency-boundary` / `dependency-boundary-violating`: Unavailable (`codeql` is unavailable or unusable in the recorded fixture environment)
- `codeql` / `reexports-aliases` / `reexport-alias-compliant`: Unavailable (`codeql` is unavailable or unusable in the recorded fixture environment)
- `codeql` / `reexports-aliases` / `reexport-alias-violating`: Unavailable (`codeql` is unavailable or unusable in the recorded fixture environment)

## Adoption cost

Clean consumer: `clean-consumer-architecture-corpus-v1` (github:OnixResearch/octet@2bdd581980eef7f6ec8363ffa20ab9f3c798c9ee).
Real consumer: `bounded-exec-44eb74e` (github:OnixResearch/bounded-exec@44eb74eb639194665ee820562d5b2ce2c3ff6a94).

| Tool | Availability | Setup steps | Config files | Clean initial | Clean incremental | Real initial | Real incremental | Incremental/initial |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `octet` | recorded | 3 | 1 | unavailable | unavailable | unavailable | unavailable | unavailable |
| `cargo-pup` | recorded | 3 | 0 | unavailable | unavailable | unavailable | unavailable | unavailable |
| `archunit-rust` | unavailable | 3 | 0 | unavailable | unavailable | unavailable | unavailable | unavailable |
| `clippy` | local | 2 | 0 | 255671526 ns | 50894103 ns | unavailable | unavailable | 1990 bp |
| `cargo-deny` | unavailable | 3 | 1 | unavailable | unavailable | unavailable | unavailable | unavailable |
| `cargo-vet` | unavailable | 3 | 1 | unavailable | unavailable | unavailable | unavailable | unavailable |
| `semgrep` | unavailable | 3 | 0 | unavailable | unavailable | unavailable | unavailable | unavailable |
| `codeql` | unavailable | 3 | 0 | unavailable | unavailable | unavailable | unavailable | unavailable |
| `repository-baseline-script` | recorded | 3 | 1 | unavailable | unavailable | unavailable | unavailable | unavailable |

- `octet` unavailable rows:
  - immutable real-consumer selection is not checked out in the recorded fixture environment
  - no reviewed adoption command; probe-only recorded tool
  - unavailable metrics: cpu_time_ns, peak_memory_bytes, network_install_time_ms
- `cargo-pup` unavailable rows:
  - immutable real-consumer selection is not checked out in the recorded fixture environment
  - no reviewed adoption command; probe-only recorded tool
  - unavailable metrics: cpu_time_ns, peak_memory_bytes, network_install_time_ms
- `archunit-rust` unavailable rows:
  - executable is unavailable in the recorded fixture environment
  - immutable real-consumer selection is not checked out in the recorded fixture environment
  - unavailable metrics: cpu_time_ns, peak_memory_bytes, network_install_time_ms
- `clippy` unavailable rows:
  - immutable real-consumer selection is not checked out in the recorded fixture environment
  - unavailable metrics: cpu_time_ns, peak_memory_bytes, network_install_time_ms
- `cargo-deny` unavailable rows:
  - executable is unavailable in the recorded fixture environment
  - immutable real-consumer selection is not checked out in the recorded fixture environment
  - unavailable metrics: cpu_time_ns, peak_memory_bytes, network_install_time_ms
- `cargo-vet` unavailable rows:
  - executable is unavailable in the recorded fixture environment
  - immutable real-consumer selection is not checked out in the recorded fixture environment
  - unavailable metrics: cpu_time_ns, peak_memory_bytes, network_install_time_ms
- `semgrep` unavailable rows:
  - executable is unavailable in the recorded fixture environment
  - immutable real-consumer selection is not checked out in the recorded fixture environment
  - unavailable metrics: cpu_time_ns, peak_memory_bytes, network_install_time_ms
- `codeql` unavailable rows:
  - executable is unavailable in the recorded fixture environment
  - immutable real-consumer selection is not checked out in the recorded fixture environment
  - unavailable metrics: cpu_time_ns, peak_memory_bytes, network_install_time_ms
- `repository-baseline-script` unavailable rows:
  - immutable real-consumer selection is not checked out in the recorded fixture environment
  - no reviewed adoption command; probe-only recorded tool
  - unavailable metrics: cpu_time_ns, peak_memory_bytes, network_install_time_ms

### Documented setup steps

- `octet` (source: docs/benchmark-evidence.md):
  - Build or install cargo-octet at the pinned source revision.
  - Export standards/benchmark/corpus/octet-architecture.ncl and pass the checked policy.
  - Select the corpus scope explicitly; do not inherit an ambient workspace.
  - required config: standards/benchmark/corpus/octet-architecture.ncl
- `cargo-pup` (source: https://github.com/DataDog/cargo-pup):
  - Install cargo-pup 0.1.9 from crates.io.
  - Declare the dependency-boundary and reexport rules in the reviewed tool configuration.
  - Run against the selected corpus targets and retain diagnostics verbatim.
- `archunit-rust` (source: https://lukasniessen.github.io/ArchUnitRust/):
  - Add archunit 0.0.2 as a test dependency in the consumer fixture.
  - Write the layer and dependency rules as a test module.
  - Run cargo test for that module and retain the test output.
- `clippy` (source: https://doc.rust-lang.org/clippy/):
  - Install the pinned Rust toolchain with the clippy component.
  - Select lints explicitly; the benchmark does not claim an architecture check.
- `cargo-deny` (source: https://embarkstudios.github.io/cargo-deny/checks/index.html):
  - Install cargo-deny 0.20.2.
  - Supply the reviewed deny.toml policy.
  - Run the dependency checks; this tool does not claim architecture or evidence assurance.
  - required config: deny.toml
- `cargo-vet` (source: https://mozilla.github.io/cargo-vet/):
  - Install cargo-vet 0.10.3.
  - Configure the audit set and retain the fetched audit state.
  - Run cargo vet; this tool does not claim architecture or evidence assurance.
  - required config: supply-chain/config.toml
- `semgrep` (source: https://semgrep.dev/docs/):
  - Install Semgrep 1.184.0.
  - Load the reviewed token-pattern rules and select the corpus path.
  - Retain SARIF or JSON output verbatim; macro and cfg outcomes remain partial.
- `codeql` (source: https://codeql.github.com/docs/):
  - Install the CodeQL CLI 2.27.4 and the Rust query pack.
  - Create the database from the corpus with the pinned compiler.
  - Run the reviewed queries and retain SARIF output verbatim.
- `repository-baseline-script` (source: standards/benchmark/assurance/repository-baseline.sh):
  - Check out the pinned repository script.
  - Provide jq and sh, then run the script against each assurance fixture.
  - Retain the exit status; an unrun script is not clean evidence.
  - required config: standards/benchmark/assurance/repository-baseline.sh

## Negative controls

| Control | Outcome | Evidence |
| --- | --- | --- |
| `noncompiling-violation-counted-as-detection` | Contained | experiment observation `octet:dependency-boundary:dependency-boundary-violating` counts a noncompiling violation as detection |
| `renamed-stale-oracle` | Contained | experiment observation `octet:dependency-boundary:dependency-boundary-compliant` has a stale oracle source or task identity |
| `differing-scope-hidden` | Contained | experiment observation `octet:dependency-boundary:dependency-boundary-compliant` analyzed a different scope than the frozen case |
| `dropped-timeouts` | Contained | experiment observation for `octet`/`cfg-violating` is missing (dropped execution or timeout) |
| `stale-results` | Contained | experiment derived comparison does not reproduce from retained inputs |
| `hidden-toolchain-cache-difference` | Contained | experiment derived comparison does not reproduce from retained inputs |
| `unsupported-counted-clean` | Contained | unsupported scope did not increase clean evidence |
| `bounded-corpus-presented-as-complete` | Contained | experiment artifact presents the bounded corpus as complete Rust coverage |

## Coverage boundary and non-claims

- Corpus: `octet-benchmark-architecture-corpus-v1`
- Bounded: `true`
- Complete Rust coverage: `false`
- The corpus is a small compiling fixture with declared compliant, violating, disputed, and empty-scope cases; it is not a complete Rust program or a representative sample of Rust projects.

- The experiment is bounded fixture evidence; it does not prove correctness, universal detection, complete Rust coverage, or release eligibility.
- The experiment does not prove correctness, universal detection, or release eligibility.
- Fixture observations are recorded evidence, not a claim that every candidate executable ran in this environment.
- Unavailable tools remain unsupported; their absence is not clean evidence.
- The bounded corpus is not complete Rust coverage and precision/recall apply only to declared comparable subsets.
