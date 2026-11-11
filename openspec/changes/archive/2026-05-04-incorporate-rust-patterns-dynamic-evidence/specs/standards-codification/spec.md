## ADDED Requirements

### Requirement: External Source Incorporation Classification [r[external-source-incorporation-classification]]

The system MUST classify each incorporated external reference concept before it becomes Tigerstyle standards guidance, policy registry data, or lint scope.

Each classification MUST name the external source, the imported concept, the Tiger Style outcome it supports, the enforcement tier, and whether the first implementation is documentation, policy metadata, a checker, or a human checklist.

#### Scenario: External concepts are classified before incorporation [r[external-source-incorporation-classification.scenario.classified-before-incorporation]]

- GIVEN a contributor proposes importing a Valgrind tool-suite concept or a Rust Design Patterns idiom
- WHEN the standards incorporation plan is updated
- THEN the plan records the source URL, selected concept, Tiger Style rationale, and enforcement tier
- AND concepts with weak static signal remain advisory or human-process-only until separate fixture evidence exists

### Requirement: Dynamic Evidence Profiles [r[dynamic-evidence-profiles]]

The system MUST document opt-in dynamic evidence profiles for bug classes that static Tigerstyle lints cannot fully prove.

The profiles MUST cover memory/allocation behavior, cache or hot-path profiling, race/concurrency analysis, unsafe or FFI boundary checking, and release-readiness timing. Each profile MUST state when to run it, the expected acceptance signal, and how evidence is saved for OpenSpec review.

#### Scenario: Dynamic profile guidance names run timing and acceptance signal [r[dynamic-evidence-profiles.scenario.timing-and-acceptance]]

- GIVEN a maintainer selects an unsafe, FFI, concurrency, or performance-sensitive slice
- WHEN the relevant dynamic evidence profile is read
- THEN the profile names the applicable run timing such as after a big change, suspected bug, pre-release, or focused CI rail
- AND it names the acceptance signal or saved transcript shape required for review

#### Scenario: Heavyweight tools remain optional until a profile is selected [r[dynamic-evidence-profiles.scenario.optional-until-selected]]

- GIVEN a normal Tigerstyle consumer runs the default lint or standards checks
- WHEN no dynamic evidence profile is selected
- THEN the default command set does not require Valgrind, sanitizers, heap profilers, cache profilers, Miri, or Loom to be installed
- AND profile docs explain how to run selected tools only for applicable evidence slices

### Requirement: Rust Idiom Guidance [r[rust-idiom-guidance]]

The system MUST provide Tigerstyle-owned Rust idiom guidance that imports only patterns that improve safety, explicitness, boundedness, recoverability, performance evidence, or developer comprehension.

The guidance MUST cover newtypes for units/capabilities/secrets, RAII resource guards, contained unsafe modules, borrowed argument types, temporary mutability, clone-to-satisfy-borrow-checker alternatives, returning consumed arguments on error, constrained builders, command/effects-as-data, FFI wrappers and error mapping, restrained `Deref` use, small-crate boundaries, and functional API patterns that use generics or sealed traits to make invalid protocol/state operations unrepresentable.

#### Scenario: Rust idiom guidance maps each imported pattern to Tiger Style outcomes [r[rust-idiom-guidance.scenario.maps-patterns-to-outcomes]]

- GIVEN a reader opens the Rust idiom guidance
- WHEN they review an imported pattern or anti-pattern
- THEN the guidance states the Tiger Style outcome it supports
- AND it distinguishes immediate documentation/checklist guidance from future lint or policy enforcement

#### Scenario: Future lints require fixtures before hard gating [r[rust-idiom-guidance.scenario.future-lints-require-fixtures]]

- GIVEN a pattern-derived lint candidate such as borrowed argument types, unsafe module containment, clone-avoidance, temporary mutability, or `Deref` restraint is proposed
- WHEN the policy registry classifies it as a hard rule
- THEN positive and negative fixtures prove the intended compliant and noncompliant shapes
- AND the lint is not promoted to a default hard gate without that fixture evidence

### Requirement: Functional API Boundary Guidance [r[functional-api-boundary-guidance]]

The system MUST document which functional programming concepts from Rust Design Patterns are acceptable Tigerstyle guidance and which remain non-goals.

The guidance MUST promote type-class-style generics, sealed traits, associated types, and marker-state APIs when they encode protocol, state, capability, or format differences at compile time. It MUST treat optics, visitors, and prism-like adapter splits as advanced human-review guidance for serialization, traversal, and plugin boundaries rather than default application style. It MUST caution against opaque point-free or deeply chained functional code in hot paths, safety-critical paths, and code that needs dense assertions.

#### Scenario: Functional API guidance promotes type-level state separation without opaque cleverness [r[functional-api-boundary-guidance.scenario.type-level-state-not-cleverness]]

- GIVEN a Tigerstyle guide imports functional programming concepts
- WHEN a reader compares type-class/generic-state APIs, optics/visitor adapter APIs, and fold-heavy declarative code
- THEN the guide recommends type-level state or protocol separation where it removes runtime `Option`/enum checks from known-state code paths
- AND it classifies optics or visitor/prism patterns as advanced boundary tools requiring human design review
- AND it warns that long iterator chains or point-free transformations are inappropriate when they hide bounds, branches, assertions, or hot-loop costs
