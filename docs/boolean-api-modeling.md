# Boolean API modeling

`raw_bool_parameter` checks direct `bool` parameters at public, crate-owned
callable boundaries: public free functions, inherent methods, and methods
declared by crate-owned public traits. It also reports non-public functions,
methods, and closures with at least two direct boolean parameters. The private
minimum is a named lint constant, not a configurable warning budget. One
private predicate flag is permitted by this rule.

The closed, human-authored Nickel `boolean_api_policy` in
`standards/strict-profiles/profiles.ncl` defines the covered shapes, private
minimum, exception boundary, and non-claims. Strict-profile validation checks
it and binds it to the selected profile identity. Do not replace its fail-closed
policy with an ad hoc workspace setting.

The caller of `deploy(true, false)` cannot see which argument selects the mode
or whether those two values describe one state. The rule reports the parameter
and callable and asks the API owner to choose a domain representation; it
cannot decide that representation from a parameter name and offers no automatic
type rewrite.

## Upstream style boundary

The [Radiance language STYLE guide](https://code.radiant.computer/radiance/tree/8c7552bb010a226d2686724e93e8e5d064ec01cdf46523185eebb9937588f7e4/STYLE)
at exact Git revision `8c7552bb010a226d2686724e93e8e5d064ec01cdf46523185eebb9937588f7e4`
is an external style reference, under the same revision's
[MIT license](https://code.radiant.computer/radiance/tree/8c7552bb010a226d2686724e93e8e5d064ec01cdf46523185eebb9937588f7e4/LICENSE).
Its BOOLEANS, CALLSITES, and TYPES sections favor named mode/role/state
choices over bare booleans, readable call sites, option records for multiple
configuration values, and one type that encodes valid state instead of
independent fields that can drift. Octet adapts only that guidance into the
bounded Rust parameter rule above; neither a type replacement nor a clean
lint result proves valid state modeling. Radiance's separate advice against
small one-use helpers and its panic/error guidance are **not** part of this
rule. This reference does not make the Radiance compiler an Octet dependency
or transfer its behavior claims to Octet.

## Choose the smallest meaningful type

- Use an **enum** when values are mutually exclusive modes or actions. For
  example, replace `publish(dry_run: bool)` with
  `publish(mode: PublishMode)`, where `PublishMode` has `Preview` and `Apply`.
  Call sites then say `publish(PublishMode::Preview)`.
- Use an **options record** when independent choices belong to one operation.
  For example, replace `deploy(force: bool, verify: bool)` with
  `deploy(options: DeployOptions)`, whose fields name those independent choices.
  Review whether all combinations are valid; an options record alone does not
  make invalid combinations unrepresentable.
- Use **typestate** when a legal operation depends on the phase of an object,
  and the type can prevent a call in the wrong phase. Do not add state machinery
  to a two-mode operation that a small enum describes more clearly.
- Use a **nominal wrapper** when a boolean has a stable domain meaning that
  should not be interchangeable with unrelated flags. Keep wire/FFI conversion
  at an explicit adapter boundary and test both conversion directions.

In each case, migrate all call sites and check observable behavior for every
old value or combination; a type-only rename is not evidence of equivalence.
For a public API, record the changed signature and migration instructions in
release notes. Do not blindly change an externally owned trait method: its
implementation signature is outside this lint because the trait owner controls
that contract. Crate-owned trait declarations are in scope.

## Exact lint boundary

Only direct `bool` parameters are counted. Boolean return values, fields,
ordinary predicates and local values, `Option<bool>`, generic aliases, and
booleans hidden inside a wrapper are not inspected by this rule. Generated
expansion output and externally owned trait implementation signatures are
excluded. A clean result does **not** prove valid state modeling, behavior,
API correctness, or compatibility with consumers beyond the checked scope.

`bool_naming` is a separate naming rule for local `let` bindings; it does not
inspect parameters. `too_many_parameters` is a separate parameter-count rule
with a default maximum of five counted parameters. Neither replaces the raw
boolean API rule. The reviewed `boolean-api-modeling` Clippy co-reporting
profile selects `clippy::fn_params_excessive_bools` as independent, warning-level
compiler evidence for excessive boolean parameters. Clippy and Octet retain
their own checker, rule, message, and finding identities; Clippy does not
cover the single public boolean case.

## Compatibility exceptions

Prefer migration. If a specific published compatibility contract cannot
change, use one narrowly scoped source expectation on the exact API with a
specific contract reason and a named owner. Do not use a crate/module/workspace-level
suppression, a lint-wide disabled setting, a finding baseline, or a warning
budget to declare this strict migration clean. Generated code and externally
owned trait signatures use the lint's explicit exclusions, not source
expectations.

The companion `raw_bool_parameter_suppression` lint rejects `allow`, broad
expectations, and expectations without an exact compatibility reason and named
owner. It governs exceptions; it is not a second parameter finding. An `allow`
on a `match` arm can propagate into a nested closure and hide a private
multiple-`bool` finding. Strict admission visits arm attributes too: direct
`raw_bool_parameter` and broad structure-group allowances, including active
`cfg_attr` forms, produce a guard error on that arm. Unrelated arm attributes
remain permitted.

The expected reason format is
`external compatibility contract: <specific contract>; owner: <named-owner>`;
the owner is one stable team or person identifier.

For example, an existing plugin ABI may fix one exported signature:

```rust
pub enum PluginAction {
    Enable,
    Disable,
}

#[expect(
    tigerstyle::raw_bool_parameter,
    reason = "external compatibility contract: plugin ABI v2 fixes this exported signature; owner: plugin-abi"
)]
pub fn legacy_plugin_entry(is_enabled: bool) -> PluginAction {
    if is_enabled {
        PluginAction::Enable
    } else {
        PluginAction::Disable
    }
}
```

The expectation belongs on that function, not its module; it must remain
reviewable and must not be copied to unrelated APIs. The example illustrates
scope and ownership, not a blanket authorization for plugin compatibility.

See [`lint-reference.md`](lint-reference.md) for lint and config identities and
[`clippy-restriction-profiles.md`](clippy-restriction-profiles.md) for the
separate Clippy rail. The bounded Octet source inventory is recorded in
[`boolean-api-migration-inventory.md`](boolean-api-migration-inventory.md).
