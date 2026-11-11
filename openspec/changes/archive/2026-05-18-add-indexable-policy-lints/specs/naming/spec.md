## ADDED Requirements

### Requirement: Underscore Module Filename Detection [r[underscore-module-filename-detection]]

The system MUST provide a warn-by-default naming lint named `underscore_in_module_filename`.

The lint MUST warn when a production module file name contains an underscore, such as `compute_vm.rs`.

The lint MUST suggest replacing compound file names with a directory hierarchy when practical, such as `compute/vm.rs`.

The lint MUST NOT warn on crate roots, generated files, fixture files, or files outside the linted package root.

#### Scenario: Compound module filename [r[underscore-module-filename-detection.scenario.compound-module-filename]]

- GIVEN a production module file at `src/compute_vm.rs`
- WHEN the lint runs
- THEN `underscore_in_module_filename` warns and suggests a directory-based module layout

#### Scenario: Directory-based module filename [r[underscore-module-filename-detection.scenario.directory-based-module-filename]]

- GIVEN production module files at `src/compute/vm.rs` and `src/compute/mod.rs`
- WHEN the lint runs
- THEN `underscore_in_module_filename` does NOT fire

### Requirement: Non-Trait Import Detection [r[non-trait-import-detection]]

The system MUST provide a warn-by-default naming/import lint named `non_trait_imports`.

The lint MUST warn on `use` imports that bring non-trait items directly into local scope.

The lint MUST allow trait imports, including trait imports renamed to `_` for method resolution.

The lint MUST allow public owner-type re-exports when the imported item is intentionally part of the module's public API.

The lint MUST NOT warn on prelude imports or generated code.

#### Scenario: Struct imported into local scope [r[non-trait-import-detection.scenario.struct-imported]]

- GIVEN production code with `use crate::addr::Addr;` followed by local uses of `Addr`
- WHEN the lint runs
- THEN `non_trait_imports` warns to prefer the qualified owner path at the use site

#### Scenario: Trait import for method resolution [r[non-trait-import-detection.scenario.trait-import]]

- GIVEN production code with `use std::io::Write as _;`
- WHEN the lint runs
- THEN `non_trait_imports` does NOT fire

#### Scenario: Public owner-type re-export [r[non-trait-import-detection.scenario.public-owner-reexport]]

- GIVEN production code with `pub use crate::addr::Addr;` that is documented as part of the module public API
- WHEN the lint runs
- THEN `non_trait_imports` does NOT fire

### Requirement: Renamed Import Detection [r[renamed-import-detection]]

The system MUST provide a warn-by-default naming/import lint named `renamed_imports`.

The lint MUST warn on `use` statements that rename an import with `as`.

The lint MUST NOT warn on `as _` trait imports used only for method resolution.

The lint MUST NOT warn when the imported item is a public re-export that preserves a documented compatibility alias.

#### Scenario: Renamed private import [r[renamed-import-detection.scenario.renamed-private-import]]

- GIVEN production code with `use crate::transport::Config as TransportConfig;`
- WHEN the lint runs
- THEN `renamed_imports` warns that the rename hides the original searchable name

#### Scenario: Trait import as underscore [r[renamed-import-detection.scenario.trait-import-underscore]]

- GIVEN production code with `use std::fmt::Write as _;`
- WHEN the lint runs
- THEN `renamed_imports` does NOT fire

#### Scenario: Documented compatibility alias [r[renamed-import-detection.scenario.compatibility-alias]]

- GIVEN production code with `pub use crate::octet::Config as SlotcarConfig;` and adjacent compatibility documentation
- WHEN the lint runs
- THEN `renamed_imports` does NOT fire

### Requirement: Path Segment Repetition Detection [r[path-segment-repetition-detection]]

The system MUST provide a warn-by-default naming lint named `path_segment_repetition`.

The lint MUST warn when an item name repeats underscore-separated words that are already present in the crate name or ancestor module path.

The lint MUST compare normalized snake-case word segments, treating hyphens in crate names as separators.

The lint MUST NOT warn when the repeated word is required to disambiguate a public compatibility name or an externally mandated trait/API name.

#### Scenario: Function repeats module name [r[path-segment-repetition-detection.scenario.function-repeats-module]]

- GIVEN production code in module `guest` that defines `fn spawn_guest_vm(...)`
- WHEN the lint runs
- THEN `path_segment_repetition` warns that `guest` is already supplied by the module path

#### Scenario: Non-repeating item name [r[path-segment-repetition-detection.scenario.non-repeating-item]]

- GIVEN production code in module `guest` that defines `fn spawn_vm(...)`
- WHEN the lint runs
- THEN `path_segment_repetition` does NOT fire

#### Scenario: Compatibility name repeats path [r[path-segment-repetition-detection.scenario.compatibility-name]]

- GIVEN production code in module `slotcar` that defines a documented compatibility item named `slotcar_config`
- WHEN the lint runs
- THEN `path_segment_repetition` does NOT fire
