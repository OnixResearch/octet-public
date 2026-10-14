use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Human,
    Json,
    Sarif,
}

impl OutputFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Human => "human",
            Self::Json => "json",
            Self::Sarif => "sarif",
        }
    }
}

#[derive(Debug)]
pub enum CommandArgs {
    Admit(AdmissionArgs),
    Artifact(ArtifactArgs),
    Check(CheckArgs),
    EvidenceIndex(EvidenceIndexArgs),
    EvidenceRail(EvidenceRailArgs),
    Fingerprint(FingerprintArgs),
    Migrate(MigrateArgs),
    Object(ObjectArgs),
    ProviderManifest(ProviderManifestArgs),
    Prove(ProveArgs),
    ProofRef(ProofRefArgs),
    Roadmap(crate::roadmap::RoadmapCommand),
    Transcript(TranscriptArgs),
    Help(HelpTopic),
    Version,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpTopic {
    Root,
    Admit,
    Artifact,
    Check,
    EvidenceIndex,
    EvidenceRail,
    Fingerprint,
    Migrate,
    Object,
    ProviderManifest,
    Prove,
    ProofRef,
    Roadmap,
    Transcript,
}

#[derive(Debug)]
pub struct AdmissionArgs {
    pub mode: AdmissionMode,
    pub artifact_dir: PathBuf,
    pub output_format: OutputFormat,
    pub command_line: String,
}

#[derive(Debug)]
pub enum AdmissionMode {
    Run {
        profile: String,
        closure: bool,
        policy: PathBuf,
        scope_args: Vec<String>,
        cargo_check_args: Option<Vec<String>>,
    },
    Verify,
}

#[derive(Debug)]
pub struct ArtifactArgs {
    pub artifact_dir: PathBuf,
    pub output_format: OutputFormat,
    pub emit_valence_provenance: Option<PathBuf>,
}

#[allow(clippy::struct_field_names)]
#[derive(Debug)]
pub struct CheckArgs {
    pub scope_args: Vec<String>,
    pub cargo_check_args: Option<Vec<String>>,
    pub profile: Option<String>,
    pub output_format: OutputFormat,
    pub artifact_dir: Option<PathBuf>,
    pub baseline: Option<PathBuf>,
    pub write_baseline: Option<PathBuf>,
    pub cache: bool,
    pub function_addresses: crate::function_address::FunctionAddressCheck,
    pub command_line: String,
}

#[derive(Debug)]
pub struct EvidenceRailArgs {
    pub rail: crate::evidence_rails::EvidenceRailKind,
    pub input: Option<PathBuf>,
    pub artifact_dir: PathBuf,
    pub workspace_root: Option<PathBuf>,
    pub config: Option<PathBuf>,
    pub baseline: Option<PathBuf>,
    pub profile: Option<String>,
    pub command: Option<String>,
    pub parent_artifacts: Vec<String>,
    pub external_reports: Vec<PathBuf>,
    pub output_format: OutputFormat,
    pub locked: bool,
    pub update_advisory_database: bool,
    pub run: bool,
    pub command_line: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceIndexMode {
    Render,
    Check,
    CodebaseBuild,
    CodebaseQuery,
    CodebaseHtml,
}

#[derive(Debug)]
pub struct EvidenceIndexArgs {
    pub mode: EvidenceIndexMode,
    pub page: Option<PathBuf>,
    pub payload_commit: Option<String>,
    pub capture_time: Option<String>,
    pub index: Option<PathBuf>,
    pub identity: Option<String>,
    pub artifacts: Vec<PathBuf>,
    pub artifact_dirs: Vec<PathBuf>,
    pub output: Option<PathBuf>,
    pub advisory: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FingerprintMode {
    Check,
    Write,
    Update,
    Diff,
}

#[derive(Debug)]
pub struct FingerprintArgs {
    pub mode: FingerprintMode,
    pub paths: Vec<PathBuf>,
    pub output_format: OutputFormat,
    pub review_manifest: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectMode {
    Emit,
    Check,
    Update,
    Diff {
        old: PathBuf,
        new: PathBuf,
        pointer_manifest: Option<PathBuf>,
    },
    PointerSet {
        manifest: PathBuf,
        name: String,
        hash: String,
        objects: PathBuf,
        receipt: PathBuf,
    },
    PointerValidate {
        manifest: PathBuf,
        objects: PathBuf,
    },
    BundleExport {
        objects: PathBuf,
        pointers: Option<PathBuf>,
        output: PathBuf,
        replay_command: String,
    },
    CorpusReceipt {
        output: PathBuf,
    },
    CorpusCheck {
        receipt: PathBuf,
    },
    CorpusCaveats,
    CorpusContract,
    SupplementEmit {
        family: SupplementFamilyArg,
        scope: SupplementScopeArg,
    },
    SupplementCheck {
        family: SupplementFamilyArg,
        scope: SupplementScopeArg,
        receipt: PathBuf,
    },
    SupplementFixtures {
        family: SupplementFamilyArg,
        scope: SupplementScopeArg,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupplementFamilyArg {
    Hir,
    Thir,
    Mir,
}

impl SupplementFamilyArg {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Hir => "hir",
            Self::Thir => "thir",
            Self::Mir => "mir",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupplementScopeArg {
    Pilot,
    RustcHirFixtureSpike,
    RustcThirFixtureSpike,
    RustcMirFixtureSpike,
}

impl SupplementScopeArg {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pilot => "pilot",
            Self::RustcHirFixtureSpike => "rustc-hir-fixture-spike",
            Self::RustcThirFixtureSpike => "rustc-thir-fixture-spike",
            Self::RustcMirFixtureSpike => "rustc-mir-fixture-spike",
        }
    }
}

#[derive(Debug)]
pub struct ObjectArgs {
    pub mode: ObjectMode,
    pub paths: Vec<PathBuf>,
    pub function: Option<String>,
    pub output_format: OutputFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrateMode {
    Start,
    Status,
    ApplySafe,
    SessionPlan,
    SessionStatus,
    SessionVerify,
    SessionApply,
}

#[derive(Debug)]
pub struct MigrateArgs {
    pub mode: MigrateMode,
    pub profile: Option<String>,
    pub output_format: OutputFormat,
    pub manifest: PathBuf,
    pub session: Option<PathBuf>,
    pub operation_manifest: Option<PathBuf>,
    pub dry_run: bool,
}

#[derive(Debug)]
pub struct TranscriptArgs {
    pub path: PathBuf,
    pub output_format: OutputFormat,
}

#[derive(Debug)]
pub struct ProviderManifestArgs {
    pub mode: ProviderManifestMode,
}

#[derive(Debug)]
pub struct ProveArgs {
    pub registry: Option<PathBuf>,
    pub policy_artifacts_dir: Option<PathBuf>,
    pub source: Vec<PathBuf>,
    pub family: Vec<String>,
    pub rail: Vec<String>,
    pub artifact_dir: PathBuf,
    pub output_format: OutputFormat,
    pub command_line: String,
    pub list: bool,
    pub explain: bool,
}

#[derive(Debug)]
pub struct ProofRefArgs {
    pub mode: ProofRefMode,
}

#[derive(Debug)]
pub enum ProofRefMode {
    Validate(ProofRefValidateArgs),
    Scaffold(ProofRefScaffoldArgs),
}

#[derive(Debug)]
pub struct ProofRefValidateArgs {
    pub proof_cards: PathBuf,
    pub reference: PathBuf,
    pub output_format: OutputFormat,
}

#[derive(Debug)]
pub struct ProofRefScaffoldArgs {
    pub proof_cards: PathBuf,
    pub proof_or_property_id: String,
    pub reference_id: String,
    pub consumer_context: String,
    pub consumer_source: PathBuf,
    pub anchor: String,
    pub output: Option<PathBuf>,
}

#[derive(Debug)]
pub enum ProviderManifestMode {
    Export {
        driver: PathBuf,
        lint_library: PathBuf,
        config: PathBuf,
        standards_policy: Option<PathBuf>,
        rustc: Option<PathBuf>,
        profile: Option<String>,
        required_lints: Vec<String>,
    },
    Validate {
        manifest: PathBuf,
    },
}

pub fn parse(argv: impl IntoIterator<Item = OsString>) -> Result<CommandArgs, String> {
    let mut argv = argv
        .into_iter()
        .map(|arg| arg.into_string().map_err(|_| "non-utf8 arguments are not supported".to_owned()))
        .collect::<Result<Vec<_>, _>>()?;

    let program = argv.first().cloned().ok_or_else(|| "missing argv[0]".to_owned())?;
    argv.remove(0);

    let invoked_as_cargo_subcommand =
        argv.first().is_some_and(|arg| matches!(arg.as_str(), "octet" | "slotcar" | "tigerstyle"));
    let command_line = if invoked_as_cargo_subcommand {
        let mut canonical_argv = argv.clone();
        canonical_argv[0].clear();
        canonical_argv[0].push_str("octet");
        format_command_line("cargo", &canonical_argv)
    } else {
        format_command_line(&program, &argv)
    };

    if invoked_as_cargo_subcommand {
        argv.remove(0);
    }

    match argv.first().map(String::as_str) {
        Some("--help" | "-h" | "help") => Ok(CommandArgs::Help(HelpTopic::Root)),
        Some("--version" | "-V" | "version") => Ok(CommandArgs::Version),
        Some("admit") if artifact_wants_help(&argv[1..]) => Ok(CommandArgs::Help(HelpTopic::Admit)),
        Some("admit") => parse_admission(&argv[1..], command_line).map(CommandArgs::Admit),
        Some("artifact") if artifact_wants_help(&argv[1..]) => Ok(CommandArgs::Help(HelpTopic::Artifact)),
        Some("artifact") => parse_artifact(&argv[1..]).map(CommandArgs::Artifact),
        Some("check") if wants_help(&argv[1..]) => Ok(CommandArgs::Help(HelpTopic::Check)),
        Some("check") => parse_check(&argv[1..], command_line).map(CommandArgs::Check),
        Some("evidence-index") if wants_help(&argv[1..]) => Ok(CommandArgs::Help(HelpTopic::EvidenceIndex)),
        Some("evidence-index") => parse_evidence_index(&argv[1..]).map(CommandArgs::EvidenceIndex),
        Some("evidence") if wants_help(&argv[1..]) => Ok(CommandArgs::Help(HelpTopic::EvidenceRail)),
        Some("evidence") => parse_evidence_rail(&argv[1..], command_line).map(CommandArgs::EvidenceRail),
        Some("fingerprint") if wants_help(&argv[1..]) => Ok(CommandArgs::Help(HelpTopic::Fingerprint)),
        Some("fingerprint") => parse_fingerprint(&argv[1..]).map(CommandArgs::Fingerprint),
        Some("migrate") if wants_help(&argv[1..]) => Ok(CommandArgs::Help(HelpTopic::Migrate)),
        Some("migrate") => parse_migrate(&argv[1..]).map(CommandArgs::Migrate),
        Some("object") if object_wants_help(&argv[1..]) => Ok(CommandArgs::Help(HelpTopic::Object)),
        Some("object") => parse_object(&argv[1..]).map(CommandArgs::Object),
        Some("provider-manifest") if wants_help(&argv[1..]) => Ok(CommandArgs::Help(HelpTopic::ProviderManifest)),
        Some("provider-manifest") => parse_provider_manifest(&argv[1..]).map(CommandArgs::ProviderManifest),
        Some("prove") if wants_help(&argv[1..]) => Ok(CommandArgs::Help(HelpTopic::Prove)),
        Some("prove") => parse_prove(&argv[1..], command_line).map(CommandArgs::Prove),
        Some("proof-ref") if wants_help(&argv[1..]) => Ok(CommandArgs::Help(HelpTopic::ProofRef)),
        Some("proof-ref") => parse_proof_ref(&argv[1..]).map(CommandArgs::ProofRef),
        Some("roadmap") if wants_help(&argv[1..]) => Ok(CommandArgs::Help(HelpTopic::Roadmap)),
        Some("roadmap") => crate::roadmap::parse(&argv[1..]).map(CommandArgs::Roadmap),
        Some("transcript") if wants_help(&argv[1..]) => Ok(CommandArgs::Help(HelpTopic::Transcript)),
        Some("transcript") => parse_transcript(&argv[1..]).map(CommandArgs::Transcript),
        Some(other) => Err(format!(
            "unknown subcommand `{other}`; expected `admit`, `artifact`, `check`, `evidence`, `evidence-index`, `fingerprint`, `migrate`, `object`, `provider-manifest`, `prove`, `proof-ref`, `roadmap`, or `transcript`"
        )),
        None => Err(
            "missing subcommand; expected `admit`, `artifact`, `check`, `evidence`, `evidence-index`, `fingerprint`, `migrate`, `object`, `provider-manifest`, `prove`, `proof-ref`, `roadmap`, or `transcript`"
                .to_owned(),
        ),
    }
}

fn wants_help(args: &[String]) -> bool {
    matches!(args.first().map(String::as_str), Some("--help" | "-h" | "help"))
}

fn artifact_wants_help(args: &[String]) -> bool {
    wants_help(args) || args.iter().any(|arg| matches!(arg.as_str(), "--help" | "-h"))
}

fn object_wants_help(args: &[String]) -> bool {
    wants_help(args) || args.iter().any(|arg| matches!(arg.as_str(), "--help" | "-h"))
}

pub fn help_text(topic: HelpTopic) -> &'static str {
    match topic {
        HelpTopic::Root => ROOT_HELP,
        HelpTopic::Admit => ADMIT_HELP,
        HelpTopic::Artifact => ARTIFACT_HELP,
        HelpTopic::Check => CHECK_HELP,
        HelpTopic::EvidenceIndex => EVIDENCE_INDEX_HELP,
        HelpTopic::EvidenceRail => EVIDENCE_RAIL_HELP,
        HelpTopic::Fingerprint => FINGERPRINT_HELP,
        HelpTopic::Migrate => MIGRATE_HELP,
        HelpTopic::Object => OBJECT_HELP,
        HelpTopic::ProviderManifest => PROVIDER_MANIFEST_HELP,
        HelpTopic::Prove => PROVE_HELP,
        HelpTopic::ProofRef => PROOF_REF_HELP,
        HelpTopic::Roadmap => ROADMAP_HELP,
        HelpTopic::Transcript => TRANSCRIPT_HELP,
    }
}

const ROOT_HELP: &str = "Octet operator commands

Usage:
  cargo octet admit --profile deterministic-core --closure [--policy PATH] [--artifact-dir PATH] [SCOPE] -- --target TARGET [CARGO_CHECK_ARGS]
  cargo octet admit verify --artifact-dir PATH [--output-format human|json]
  cargo octet artifact verify --artifact-dir PATH [--output-format human|json] [--emit-valence-provenance PATH]
  cargo octet check [SCOPE] [--profile robust|bounded-systems|deterministic-core] [--function-addresses off|observe|warn|deny] [--function-address-path PATH] [-- CARGO_CHECK_ARGS]
  cargo octet evidence collect --rail RAIL --input PATH --artifact-dir PATH [--output-format human|json]
  cargo octet evidence-index render --payload-commit HASH --capture-time RFC3339
  cargo octet evidence-index check [--page PATH]
  cargo octet evidence-index codebase build --artifact PATH [--artifact-dir DIR] [--output PATH] [--advisory]
  cargo octet evidence-index codebase query --index PATH --identity ID
  cargo octet evidence-index codebase html --index PATH [--output PATH]
  cargo octet fingerprint [--check|--update|--diff] [PATHS...]
  cargo octet migrate <start|status|apply-safe> [--manifest PATH]
  cargo octet migrate session plan|status|verify --session PATH [--output-format human|json]
  cargo octet migrate session apply --session PATH --operation-manifest PATH [--dry-run]
  cargo octet object emit|check|update [--function NAME] [PATHS...]
  cargo octet object diff --old OLD.json --new NEW.json [--pointer-manifest PATH]
  cargo octet object pointer set --manifest PATH --name NAME --hash HASH --objects OBJECTS.json --receipt PATH
  cargo octet object pointer validate --manifest PATH --objects OBJECTS.json
  cargo octet object bundle export --objects OBJECTS.json [--pointers POINTERS.json] --output BUNDLE.json --replay-command CMD
  cargo octet object corpus receipt --output RECEIPT.json [PATHS...]
  cargo octet object corpus check --receipt RECEIPT.json [PATHS...]
  cargo octet object corpus caveats [--output-format human|json] [PATHS...]
  cargo octet object corpus contract [--output-format human|json]
  cargo octet object supplement fixtures --family hir --spike rustc-hir-fixture [--output-format human|json]
  cargo octet object supplement fixtures --family thir --spike rustc-thir-fixture [--output-format human|json]
  cargo octet object supplement fixtures --family mir --spike rustc-mir-fixture [--output-format human|json]
  cargo octet object supplement emit --family hir|thir|mir [--spike rustc-hir-fixture|rustc-thir-fixture|rustc-mir-fixture] [--fixture-shape checked-arithmetic|branch-control-flow|call-return] [--function NAME] [PATHS...]
  cargo octet object supplement check --family hir|thir|mir [--spike rustc-hir-fixture|rustc-thir-fixture|rustc-mir-fixture] [--fixture-shape checked-arithmetic|branch-control-flow|call-return] --receipt RECEIPT.json [--function NAME] [PATHS...]
  cargo octet provider-manifest export --driver PATH --lint-library PATH --config PATH [--standards-policy PATH]
  cargo octet provider-manifest validate --manifest PATH
  cargo octet prove --policy-artifacts-dir PATH --source PATH --artifact-dir PATH [--rail POLICY[#rail-N]] [--list] [--explain] [--output-format human|json]
  cargo octet proof-ref validate --proof-cards PATH --ref PATH [--output-format human|json]
  cargo octet proof-ref scaffold --proof-cards PATH --proof-or-property-id ID --reference-id ID --consumer-context TEXT --consumer-source PATH --anchor TEXT [--output PATH]
  cargo octet roadmap <artifact|capsule|ability|pointer|session|pure-cache> ...
  cargo octet transcript <PATH>
  cargo octet --version

Release smoke surfaces: fingerprint identity review, migrate manifests, check --cache lint-cache status, effect-summary output in summaries/artifacts, and executable transcript validation.

Nix: nix run path:/path/to/octet#cargo-octet -- check
";

const ADMIT_HELP: &str = "cargo octet admit

Commands:
  cargo octet admit --profile deterministic-core --closure [--policy octet-effect-closure.json] [--artifact-dir target/octet-admission] [SCOPE] -- --target TARGET [CARGO_CHECK_ARGS]
  cargo octet admit verify --artifact-dir PATH [--output-format human|json]

Runs the existing deterministic-core check first, then computes conservative selected source-token effect closure under exact reviewed call summaries. An explicit Cargo target is required to exclude ambient target ambiguity. Unknown effects, opaque constructs, stale function identities, forbidden role edges, and unadmitted or identity-mismatched calls fail closed. Verification checks stored BLAKE3 artifact integrity without rerunning Cargo.

V1 does not claim complete Rust name resolution, macro or trait-dispatch completeness, MIR call-graph completeness, dependency implementation verification, runtime reachability or ordering, panic freedom, semantic determinism, whole-program correctness, compiler correctness, or release eligibility.
";

const ARTIFACT_HELP: &str = "cargo octet artifact verify

Options: --artifact-dir PATH, --output-format human|json, --emit-valence-provenance PATH.

Verifies a saved cargo octet check/prove artifact directory or a log-only evidence-log-manifest.json directory without mutating it. The verifier recomputes BLAKE3 hashes, validates provenance coverage, verifies raw log artifacts, rejects stale sidecars, and reports deterministic human or JSON output. The optional Valence provenance export is written only after verification succeeds.
";

const CHECK_HELP: &str = "cargo octet check

Options: --profile robust|bounded-systems|deterministic-core, --output-format human|json|sarif, --artifact-dir PATH, --baseline PATH, --write-baseline PATH, --cache, --function-addresses off|observe|warn|deny, --function-address-path PATH, --function-address-baseline PATH.

Strict profiles activate reviewed deny-level lint floors and minimum function-address modes. They validate direct source-shape policy only; they do not prove transitive effects, whole-program correctness, or release eligibility. Artifacts include summary.txt effect-summary output, findings JSON/SARIF, cache_status fields when --cache enables the lint-cache, and optional function-address diagnostics for gradual content-address adoption.
";

const EVIDENCE_INDEX_HELP: &str = "cargo octet evidence-index

Subcommands:
  render --payload-commit HASH --capture-time RFC3339
  check [--page docs/current-head-valence-evidence.md]
  codebase build --artifact PATH [--artifact-dir DIR] [--output PATH] [--advisory]
  codebase query --index PATH --identity ID
  codebase html --index PATH [--output PATH]

Renders or checks the generated current-head Valence evidence Markdown index. Check mode fails closed on stale payload, missing rail/pass signal, or weakened non-claim boundaries.

The codebase subcommands build a deterministic, content-addressed review index over existing Octet evidence artifacts, query it by stable identity, and render static local HTML. The generated index proves artifact identity and linkage only; it is not a correctness, semantic-equivalence, complete-dependency-closure, distributed-execution, or remote-code-mobility proof.
";

const EVIDENCE_RAIL_HELP: &str = "cargo octet evidence collect

Options: --rail dependency-audit|clippy-co-reporting|ast-grep|wasm-artifact|charon-semantic|dependency-trust|api-stability|dynamic-evidence|sarif-import|release-supply-chain|mutation-testing|coverage-baseline|kani-model-check|hegel-property|z4-solver|verifier-fixture, --input PATH or --run, --artifact-dir PATH, --workspace-root PATH, --config PATH, --baseline PATH, --profile NAME, --command COMMAND, --parent-artifact PATH, --external-report PATH, --locked, --update-advisory-database, --output-format human|json.

Collects optional non-starter evidence from external tools or saved tool output into a Rust-owned evidence-rail receipt, summary, and provenance bundle. Clippy run mode selects the shipped checked restriction catalog with `--profile production-safety|canonical-determinism|shell-boundary-inventory`; `--config PATH` may supply another checked catalog. It constructs fixed workspace/all-target argv and rejects arbitrary command authority; the blanket `clippy::restriction` group is unsupported. The ast-grep run mode requires a checked generated profile and rejects arbitrary command authority. The wasm-artifact rail requires exact `--input` bytes plus `--config` and `--profile`, never executes the artifact, and embeds JSON/SARIF findings in a self-verifiable exact-byte bundle. SpaceWasm profiles may link identity-verified Mantle, ChaosControl, or Molten observations with repeated `--external-report`; linked observations remain recorded-only and cannot change static conformance. The charon-semantic rail requires the shipped profile, reuses the pinned network-denied Charon stage, rejects arbitrary command authority, and records optional source/token comparisons without merging authority. The `cargo-fuzz-libfuzzer` dynamic profile binds the generated Nickel registry, fixed argv, bounded corpus identities, crash roles, replay argv, and transcript-only non-claims; it rejects arbitrary `--command` input. The `sarif-import` rail accepts only checked local producer profiles and preserves raw and normalized identities without running a scanner. Hegel run mode requires the checked `octet-pure-core-laws-v1` producer profile, uses a fixed command and dedicated bounded summary, and rejects `--command`. Legacy Hegel v1 import remains available. Z4 and verifier-fixture run modes require explicit commands and never bootstrap their tools automatically. These rails are complementary to cargo octet check; they do not upgrade validation evidence into formal proof.
";

const FINGERPRINT_HELP: &str = "cargo octet fingerprint

Modes: --check, --write, --update, --diff. Use --review-manifest PATH to save review records. Fingerprint receipts use stable semantic identity and BLAKE3 hashes.
";

const MIGRATE_HELP: &str = "cargo octet migrate

Subcommands: start, status, apply-safe, session plan, session status, session verify, session apply.

Migration manifests and structured migration sessions are deterministic and safe-reviewable before applying. Session plan/status/verify are non-mutating; session apply writes a pre-write operation manifest for selected safe record-only operations and preserves non-claims about semantic equivalence and correctness.
";

const OBJECT_HELP: &str = "cargo octet object

Subcommands:
  emit [--function NAME] [PATHS...] --output-format human|json
  check [--function NAME] [PATHS...] --output-format human|json
  update [--function NAME] [PATHS...] --output-format human|json
  diff --old OLD.json --new NEW.json [--pointer-manifest PATH] --output-format human|json
  pointer set --manifest PATH --name NAME --hash HASH --objects OBJECTS.json --receipt PATH
  pointer validate --manifest PATH --objects OBJECTS.json
  bundle export --objects OBJECTS.json [--pointers POINTERS.json] --output BUNDLE.json --replay-command CMD
  corpus receipt --output RECEIPT.json [PATHS...]
  corpus check --receipt RECEIPT.json [PATHS...] --output-format human|json
  corpus caveats [PATHS...] --output-format human|json
  corpus contract --output-format human|json
  supplement fixtures --family hir --spike rustc-hir-fixture --output-format human|json
  supplement fixtures --family thir --spike rustc-thir-fixture --output-format human|json
  supplement fixtures --family mir --spike rustc-mir-fixture --output-format human|json
  supplement emit --family hir|thir|mir [--spike rustc-hir-fixture|rustc-thir-fixture|rustc-mir-fixture] [--fixture-shape checked-arithmetic|branch-control-flow|call-return] [--function NAME] [PATHS...] --output-format human|json
  supplement check --family hir|thir|mir [--spike rustc-hir-fixture|rustc-thir-fixture|rustc-mir-fixture] [--fixture-shape checked-arithmetic|branch-control-flow|call-return] --receipt RECEIPT.json [--function NAME] [PATHS...] --output-format human|json

Function objects are source-token review/provenance evidence. HIR/MIR supplements attach to v1 identity; THIR default supplements are fail-closed unsupported-planned evidence, with only selected rustc-thir-fixture and rustc-mir-fixture cases reporting pilot-supported. Supplements are not correctness proofs.
";

const PROVIDER_MANIFEST_HELP: &str = "cargo octet provider-manifest

Subcommands:
  export --driver PATH --lint-library PATH --config PATH [--standards-policy PATH] [--rustc PATH] [--profile NAME] [--required-lint NAME]
  validate --manifest PATH

Exports or validates the deterministic Octet compiler-policy provider manifest consumed by downstream build systems such as Mantle. The manifest identifies the Dylint driver, Octet lint library, lint config, optional standards policy artifact, compatible rustc identity, required environment, waiver contract, and bounded non-claims.
";

const PROVE_HELP: &str = "cargo octet prove

Options: --registry PATH or --policy-artifacts-dir PATH, --source PATH (repeatable), --family NAME, --rail POLICY[#rail-N], --artifact-dir PATH, --list, --explain, --output-format human|json.

Runs selected standards proof rails through argv-based commands, writes structured proof receipts under the artifact directory, emits formal_proof provenance only for validated receipts with verifier identity and proof artifact identity, and keeps ordinary cargo octet check behavior unchanged. Use --rail to narrow selection to one policy or property ID, --list to show selected rails without executing them, and --explain to prefix failure diagnostics with remediation context.
";

const PROOF_REF_HELP: &str = "cargo octet proof-ref

Commands:
  validate --proof-cards PATH --ref PATH [--output-format human|json]
  scaffold --proof-cards PATH --proof-or-property-id ID --reference-id ID --consumer-context TEXT --consumer-source PATH --anchor TEXT [--output PATH]

Validates a Rust-owned ProofEvidenceRef JSON record against a saved proof-cards.json deck and, when the reference names consumer_source_path/consumer_source_blake3/consumer_source_anchor fields, checks that source/comment anchor too. Scaffold reads a proof card and anchored consumer source to emit ready JSON without rerunning Verus. This command set does not emit formal_proof provenance and does not introduce a consumer-implementable Verified trait.
";

const ROADMAP_HELP: &str = "cargo octet roadmap

Unison-inspired agent roadmap fixtures: content-addressed artifacts, ability manifests, name-to-hash pointers, typed session records, structured todos, and pure-result cache receipts.

Actions:
  artifact store|inspect|replay
  capsule capture|inspect|replay
  ability registry|summary|enforce
  pointer set|inspect|rollback
  session check|migrate|todo
  pure-cache check
";

const TRANSCRIPT_HELP: &str = "cargo octet transcript <PATH>

Validates executable transcript command blocks and expected-output assertions for docs-example and release smoke artifacts.
";

fn parse_admission(args: &[String], command_line: String) -> Result<AdmissionArgs, String> {
    const DEFAULT_ARTIFACT_DIR: &str = "target/octet-admission";
    const DEFAULT_POLICY_PATH: &str = crate::effect_closure::WORKSPACE_POLICY_FILE;
    const FLAG_AND_VALUE_ARGS: usize = 2;

    if args.first().is_some_and(|arg| arg == "verify") {
        return parse_admission_verify(&args[1..], command_line);
    }

    let mut profile = None;
    let mut closure = false;
    let mut policy = PathBuf::from(DEFAULT_POLICY_PATH);
    let mut artifact_dir = PathBuf::from(DEFAULT_ARTIFACT_DIR);
    let mut output_format = OutputFormat::Human;
    let mut scope_args = Vec::new();
    let mut cargo_check_args = None;
    let mut index = 0;

    while index < args.len() {
        let arg = &args[index];
        if arg == "--" {
            cargo_check_args = Some(args[index + 1..].to_vec());
            break;
        }
        match arg.as_str() {
            "--profile" => {
                if profile.is_some() {
                    return Err("`--profile` may be provided only once".to_owned());
                }
                profile = Some(required_arg_at(args, index, arg)?);
                index += FLAG_AND_VALUE_ARGS;
            }
            "--closure" => {
                if closure {
                    return Err("`--closure` may be provided only once".to_owned());
                }
                closure = true;
                index += 1;
            }
            "--policy" => {
                policy = PathBuf::from(required_arg_at(args, index, arg)?);
                index += FLAG_AND_VALUE_ARGS;
            }
            "--artifact-dir" => {
                artifact_dir = PathBuf::from(required_arg_at(args, index, arg)?);
                index += FLAG_AND_VALUE_ARGS;
            }
            "--output-format" | "--format" => {
                output_format = parse_output_format(&required_arg_at(args, index, arg)?)?;
                index += FLAG_AND_VALUE_ARGS;
            }
            "-p" | "--package" | "--exclude" => {
                scope_args.push(arg.clone());
                scope_args.push(required_arg_at(args, index, arg)?);
                index += FLAG_AND_VALUE_ARGS;
            }
            "--workspace" => {
                scope_args.push(arg.clone());
                index += 1;
            }
            _ if arg.starts_with("--profile=") => {
                if profile.is_some() {
                    return Err("`--profile` may be provided only once".to_owned());
                }
                let value = arg.trim_start_matches("--profile=");
                if value.is_empty() {
                    return Err("missing value after `--profile`".to_owned());
                }
                profile = Some(value.to_owned());
                index += 1;
            }
            _ if arg.starts_with("--policy=") => {
                policy = PathBuf::from(arg.trim_start_matches("--policy="));
                index += 1;
            }
            _ if arg.starts_with("--artifact-dir=") => {
                artifact_dir = PathBuf::from(arg.trim_start_matches("--artifact-dir="));
                index += 1;
            }
            _ if arg.starts_with("--output-format=") => {
                output_format = parse_output_format(arg.trim_start_matches("--output-format="))?;
                index += 1;
            }
            _ if arg.starts_with("--format=") => {
                output_format = parse_output_format(arg.trim_start_matches("--format="))?;
                index += 1;
            }
            _ => return Err(format!("unsupported admission argument `{arg}`")),
        }
    }

    let profile =
        profile.ok_or_else(|| "effect closure admission requires `--profile deterministic-core`".to_owned())?;
    if !closure {
        return Err("effect closure admission requires explicit `--closure`".to_owned());
    }
    if output_format == OutputFormat::Sarif {
        return Err("effect closure admission supports human or json output, not SARIF".to_owned());
    }

    Ok(AdmissionArgs {
        mode: AdmissionMode::Run {
            profile,
            closure,
            policy,
            scope_args,
            cargo_check_args,
        },
        artifact_dir,
        output_format,
        command_line,
    })
}

fn parse_admission_verify(args: &[String], command_line: String) -> Result<AdmissionArgs, String> {
    const FLAG_AND_VALUE_ARGS: usize = 2;

    let mut artifact_dir = None;
    let mut output_format = OutputFormat::Human;
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "--artifact-dir" => {
                artifact_dir = Some(PathBuf::from(required_arg_at(args, index, arg)?));
                index += FLAG_AND_VALUE_ARGS;
            }
            "--output-format" | "--format" => {
                output_format = parse_output_format(&required_arg_at(args, index, arg)?)?;
                index += FLAG_AND_VALUE_ARGS;
            }
            _ if arg.starts_with("--artifact-dir=") => {
                artifact_dir = Some(PathBuf::from(arg.trim_start_matches("--artifact-dir=")));
                index += 1;
            }
            _ if arg.starts_with("--output-format=") => {
                output_format = parse_output_format(arg.trim_start_matches("--output-format="))?;
                index += 1;
            }
            _ if arg.starts_with("--format=") => {
                output_format = parse_output_format(arg.trim_start_matches("--format="))?;
                index += 1;
            }
            _ => return Err(format!("unsupported admission verification argument `{arg}`")),
        }
    }
    if output_format == OutputFormat::Sarif {
        return Err("effect closure admission verification supports human or json output, not SARIF".to_owned());
    }
    Ok(AdmissionArgs {
        mode: AdmissionMode::Verify,
        artifact_dir: artifact_dir.ok_or_else(|| "admission verification requires `--artifact-dir PATH`".to_owned())?,
        output_format,
        command_line,
    })
}

fn parse_artifact(args: &[String]) -> Result<ArtifactArgs, String> {
    match args.first().map(String::as_str) {
        Some("verify") => parse_artifact_verify(&args[1..]),
        Some(other) => Err(format!("unknown artifact subcommand `{other}`; expected `verify`")),
        None => Err("missing artifact subcommand; expected `verify`".to_owned()),
    }
}

fn parse_artifact_verify(args: &[String]) -> Result<ArtifactArgs, String> {
    const FLAG_AND_VALUE_ARGS: usize = 2;

    let mut artifact_dir = None;
    let mut output_format = OutputFormat::Human;
    let mut emit_valence_provenance = None;
    let mut index = 0;

    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "--artifact-dir" => {
                let value = args.get(index + 1).ok_or_else(|| "missing value after `--artifact-dir`".to_owned())?;
                artifact_dir = Some(PathBuf::from(value));
                index += FLAG_AND_VALUE_ARGS;
            }
            "--output-format" | "--format" => {
                let value = args.get(index + 1).ok_or_else(|| format!("missing value after `{arg}`"))?;
                output_format = parse_output_format(value)?;
                index += FLAG_AND_VALUE_ARGS;
            }
            "--emit-valence-provenance" => {
                let value =
                    args.get(index + 1).ok_or_else(|| "missing value after `--emit-valence-provenance`".to_owned())?;
                emit_valence_provenance = Some(PathBuf::from(value));
                index += FLAG_AND_VALUE_ARGS;
            }
            _ if arg.starts_with("--artifact-dir=") => {
                artifact_dir = Some(PathBuf::from(arg.trim_start_matches("--artifact-dir=")));
                index += 1;
            }
            _ if arg.starts_with("--output-format=") => {
                output_format = parse_output_format(arg.trim_start_matches("--output-format="))?;
                index += 1;
            }
            _ if arg.starts_with("--format=") => {
                output_format = parse_output_format(arg.trim_start_matches("--format="))?;
                index += 1;
            }
            _ if arg.starts_with("--emit-valence-provenance=") => {
                emit_valence_provenance = Some(PathBuf::from(arg.trim_start_matches("--emit-valence-provenance=")));
                index += 1;
            }
            _ if arg.starts_with('-') => return Err(format!("unsupported artifact verify argument `{arg}`")),
            _ => return Err(format!("unexpected artifact verify path `{arg}`; use `--artifact-dir`")),
        }
    }

    Ok(ArtifactArgs {
        artifact_dir: artifact_dir.ok_or_else(|| "artifact verify requires `--artifact-dir PATH`".to_owned())?,
        output_format,
        emit_valence_provenance,
    })
}

fn parse_check(args: &[String], command_line: String) -> Result<CheckArgs, String> {
    let mut scope_args = Vec::new();
    let mut cargo_check_args = None;
    let mut profile = None;
    let mut output_format = OutputFormat::Human;
    let mut artifact_dir = None;
    let mut baseline = None;
    let mut write_baseline = None;
    let mut cache = false;
    let mut function_address_mode = crate::function_address::FunctionAddressMode::Off;
    let mut function_address_paths = Vec::new();
    let mut function_address_baseline = None;
    let mut index = 0;

    while index < args.len() {
        let arg = &args[index];
        if arg == "--" {
            cargo_check_args = Some(args[index + 1..].to_vec());
            break;
        }

        if arg == "--profile" {
            if profile.is_some() {
                return Err("`--profile` may be provided only once".to_owned());
            }
            let value = args.get(index + 1).ok_or_else(|| "missing value after `--profile`".to_owned())?;
            profile = Some(value.clone());
            index += 2;
        } else if let Some(value) = arg.strip_prefix("--profile=") {
            if profile.is_some() {
                return Err("`--profile` may be provided only once".to_owned());
            }
            if value.is_empty() {
                return Err("missing value after `--profile`".to_owned());
            }
            profile = Some(value.to_owned());
            index += 1;
        } else if arg == "--output-format" {
            let value = args.get(index + 1).ok_or_else(|| "missing value after `--output-format`".to_owned())?;
            output_format = parse_output_format(value)?;
            index += 2;
        } else if let Some(value) = arg.strip_prefix("--output-format=") {
            output_format = parse_output_format(value)?;
            index += 1;
        } else if arg == "--artifact-dir" {
            let value = args.get(index + 1).ok_or_else(|| "missing value after `--artifact-dir`".to_owned())?;
            artifact_dir = Some(PathBuf::from(value));
            index += 2;
        } else if let Some(value) = arg.strip_prefix("--artifact-dir=") {
            artifact_dir = Some(PathBuf::from(value));
            index += 1;
        } else if arg == "--baseline" {
            let value = args.get(index + 1).ok_or_else(|| "missing value after `--baseline`".to_owned())?;
            baseline = Some(PathBuf::from(value));
            index += 2;
        } else if let Some(value) = arg.strip_prefix("--baseline=") {
            baseline = Some(PathBuf::from(value));
            index += 1;
        } else if arg == "--cache" {
            cache = true;
            index += 1;
        } else if arg == "--function-addresses" {
            let value = args.get(index + 1).ok_or_else(|| "missing value after `--function-addresses`".to_owned())?;
            function_address_mode = crate::function_address::FunctionAddressMode::parse(value)?;
            index += 2;
        } else if let Some(value) = arg.strip_prefix("--function-addresses=") {
            function_address_mode = crate::function_address::FunctionAddressMode::parse(value)?;
            index += 1;
        } else if arg == "--function-address-path" {
            let value =
                args.get(index + 1).ok_or_else(|| "missing value after `--function-address-path`".to_owned())?;
            function_address_paths.push(PathBuf::from(value));
            index += 2;
        } else if let Some(value) = arg.strip_prefix("--function-address-path=") {
            function_address_paths.push(PathBuf::from(value));
            index += 1;
        } else if arg == "--function-address-baseline" {
            let value =
                args.get(index + 1).ok_or_else(|| "missing value after `--function-address-baseline`".to_owned())?;
            function_address_baseline = Some(PathBuf::from(value));
            index += 2;
        } else if let Some(value) = arg.strip_prefix("--function-address-baseline=") {
            function_address_baseline = Some(PathBuf::from(value));
            index += 1;
        } else if arg == "--write-baseline" {
            let value = args.get(index + 1).ok_or_else(|| "missing value after `--write-baseline`".to_owned())?;
            write_baseline = Some(PathBuf::from(value));
            index += 2;
        } else if let Some(value) = arg.strip_prefix("--write-baseline=") {
            write_baseline = Some(PathBuf::from(value));
            index += 1;
        } else {
            match arg.as_str() {
                "-p" | "--package" | "--exclude" => {
                    let value = args.get(index + 1).ok_or_else(|| format!("missing value after `{arg}`"))?;
                    scope_args.push(arg.clone());
                    scope_args.push(value.clone());
                    index += 2;
                }
                "--workspace" => {
                    scope_args.push(arg.clone());
                    index += 1;
                }
                other => return Err(format!("unsupported argument before `--`: `{other}`")),
            }
        }
    }

    Ok(CheckArgs {
        scope_args,
        cargo_check_args,
        profile,
        output_format,
        artifact_dir,
        baseline,
        write_baseline,
        cache,
        function_addresses: crate::function_address::FunctionAddressCheck {
            mode: function_address_mode,
            paths: function_address_paths,
            baseline: function_address_baseline,
        },
        command_line,
    })
}

fn parse_evidence_index(args: &[String]) -> Result<EvidenceIndexArgs, String> {
    match args.first().map(String::as_str) {
        Some("render") => parse_valence_evidence_index(EvidenceIndexMode::Render, &args[1..]),
        Some("check") => parse_valence_evidence_index(EvidenceIndexMode::Check, &args[1..]),
        Some("codebase") => parse_codebase_evidence_index(&args[1..]),
        Some(other) => {
            Err(format!("unknown evidence-index subcommand `{other}`; expected `render`, `check`, or `codebase`"))
        }
        None => Err("missing evidence-index subcommand; expected `render`, `check`, or `codebase`".to_owned()),
    }
}

fn parse_valence_evidence_index(mode: EvidenceIndexMode, args: &[String]) -> Result<EvidenceIndexArgs, String> {
    const FLAG_AND_VALUE_ARGS: usize = 2;

    let mut page = None;
    let mut payload_commit = None;
    let mut capture_time = None;
    let mut index = 0;

    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "--page" => {
                page = Some(PathBuf::from(required_arg_at(args, index, arg)?));
                index += FLAG_AND_VALUE_ARGS;
            }
            "--payload-commit" => {
                payload_commit = Some(required_arg_at(args, index, arg)?);
                index += FLAG_AND_VALUE_ARGS;
            }
            "--capture-time" => {
                capture_time = Some(required_arg_at(args, index, arg)?);
                index += FLAG_AND_VALUE_ARGS;
            }
            _ if arg.starts_with("--page=") => {
                page = Some(PathBuf::from(arg.trim_start_matches("--page=")));
                index += 1;
            }
            _ if arg.starts_with("--payload-commit=") => {
                payload_commit = Some(arg.trim_start_matches("--payload-commit=").to_owned());
                index += 1;
            }
            _ if arg.starts_with("--capture-time=") => {
                capture_time = Some(arg.trim_start_matches("--capture-time=").to_owned());
                index += 1;
            }
            _ => return Err(format!("unsupported evidence-index argument `{arg}`")),
        }
    }

    Ok(EvidenceIndexArgs {
        mode,
        page,
        payload_commit,
        capture_time,
        index: None,
        identity: None,
        artifacts: Vec::new(),
        artifact_dirs: Vec::new(),
        output: None,
        advisory: false,
    })
}

fn parse_codebase_evidence_index(args: &[String]) -> Result<EvidenceIndexArgs, String> {
    let mode = match args.first().map(String::as_str) {
        Some("build") => EvidenceIndexMode::CodebaseBuild,
        Some("query") => EvidenceIndexMode::CodebaseQuery,
        Some("html") => EvidenceIndexMode::CodebaseHtml,
        Some(other) => {
            return Err(format!(
                "unknown evidence-index codebase action `{other}`; expected `build`, `query`, or `html`"
            ));
        }
        None => return Err("missing evidence-index codebase action; expected `build`, `query`, or `html`".to_owned()),
    };

    const FLAG_AND_VALUE_ARGS: usize = 2;
    let mut index_path = None;
    let mut identity = None;
    let mut artifacts = Vec::new();
    let mut artifact_dirs = Vec::new();
    let mut output = None;
    let mut advisory = false;
    let mut cursor = 1;

    while cursor < args.len() {
        let arg = &args[cursor];
        match arg.as_str() {
            "--index" => {
                index_path = Some(PathBuf::from(required_arg_at(args, cursor, arg)?));
                cursor += FLAG_AND_VALUE_ARGS;
            }
            "--identity" => {
                identity = Some(required_arg_at(args, cursor, arg)?);
                cursor += FLAG_AND_VALUE_ARGS;
            }
            "--artifact" => {
                artifacts.push(PathBuf::from(required_arg_at(args, cursor, arg)?));
                cursor += FLAG_AND_VALUE_ARGS;
            }
            "--artifact-dir" => {
                artifact_dirs.push(PathBuf::from(required_arg_at(args, cursor, arg)?));
                cursor += FLAG_AND_VALUE_ARGS;
            }
            "--output" => {
                output = Some(PathBuf::from(required_arg_at(args, cursor, arg)?));
                cursor += FLAG_AND_VALUE_ARGS;
            }
            "--advisory" => {
                advisory = true;
                cursor += 1;
            }
            _ if arg.starts_with("--index=") => {
                index_path = Some(PathBuf::from(arg.trim_start_matches("--index=")));
                cursor += 1;
            }
            _ if arg.starts_with("--identity=") => {
                identity = Some(arg.trim_start_matches("--identity=").to_owned());
                cursor += 1;
            }
            _ if arg.starts_with("--artifact=") => {
                artifacts.push(PathBuf::from(arg.trim_start_matches("--artifact=")));
                cursor += 1;
            }
            _ if arg.starts_with("--artifact-dir=") => {
                artifact_dirs.push(PathBuf::from(arg.trim_start_matches("--artifact-dir=")));
                cursor += 1;
            }
            _ if arg.starts_with("--output=") => {
                output = Some(PathBuf::from(arg.trim_start_matches("--output=")));
                cursor += 1;
            }
            _ => return Err(format!("unsupported evidence-index codebase argument `{arg}`")),
        }
    }

    match mode {
        EvidenceIndexMode::CodebaseBuild if artifacts.is_empty() && artifact_dirs.is_empty() => {
            return Err(
                "evidence-index codebase build requires at least one `--artifact` or `--artifact-dir`".to_owned()
            );
        }
        EvidenceIndexMode::CodebaseQuery if index_path.is_none() || identity.is_none() => {
            return Err("evidence-index codebase query requires `--index PATH --identity ID`".to_owned());
        }
        EvidenceIndexMode::CodebaseHtml if index_path.is_none() => {
            return Err("evidence-index codebase html requires `--index PATH`".to_owned());
        }
        _ => {}
    }

    Ok(EvidenceIndexArgs {
        mode,
        page: None,
        payload_commit: None,
        capture_time: None,
        index: index_path,
        identity,
        artifacts,
        artifact_dirs,
        output,
        advisory,
    })
}

fn parse_evidence_rail(args: &[String], command_line: String) -> Result<EvidenceRailArgs, String> {
    match args.first().map(String::as_str) {
        Some("collect") => parse_evidence_collect(&args[1..], command_line),
        Some(other) => Err(format!("unknown evidence subcommand `{other}`; expected `collect`")),
        None => Err("missing evidence subcommand; expected `collect`".to_owned()),
    }
}

fn parse_evidence_collect(args: &[String], command_line: String) -> Result<EvidenceRailArgs, String> {
    const FLAG_AND_VALUE_ARGS: usize = 2;

    let mut rail = None;
    let mut input = None;
    let mut artifact_dir = None;
    let mut workspace_root = None;
    let mut config = None;
    let mut baseline = None;
    let mut profile = None;
    let mut command = None;
    let mut parent_artifacts = Vec::new();
    let mut external_reports = Vec::new();
    let mut output_format = OutputFormat::Human;
    let mut locked = false;
    let mut update_advisory_database = false;
    let mut run = false;
    let mut index = 0;

    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "--rail" => {
                rail =
                    Some(crate::evidence_rails::EvidenceRailKind::parse(required_arg_at(args, index, arg)?.as_str())?);
                index += FLAG_AND_VALUE_ARGS;
            }
            "--input" => {
                input = Some(PathBuf::from(required_arg_at(args, index, arg)?));
                index += FLAG_AND_VALUE_ARGS;
            }
            "--artifact-dir" => {
                artifact_dir = Some(PathBuf::from(required_arg_at(args, index, arg)?));
                index += FLAG_AND_VALUE_ARGS;
            }
            "--workspace-root" => {
                workspace_root = Some(PathBuf::from(required_arg_at(args, index, arg)?));
                index += FLAG_AND_VALUE_ARGS;
            }
            "--config" => {
                config = Some(PathBuf::from(required_arg_at(args, index, arg)?));
                index += FLAG_AND_VALUE_ARGS;
            }
            "--baseline" => {
                baseline = Some(PathBuf::from(required_arg_at(args, index, arg)?));
                index += FLAG_AND_VALUE_ARGS;
            }
            "--profile" => {
                profile = Some(required_arg_at(args, index, arg)?);
                index += FLAG_AND_VALUE_ARGS;
            }
            "--command" => {
                command = Some(required_arg_at(args, index, arg)?);
                index += FLAG_AND_VALUE_ARGS;
            }
            "--parent-artifact" => {
                parent_artifacts.push(required_arg_at(args, index, arg)?);
                index += FLAG_AND_VALUE_ARGS;
            }
            "--external-report" => {
                external_reports.push(PathBuf::from(required_arg_at(args, index, arg)?));
                index += FLAG_AND_VALUE_ARGS;
            }
            "--output-format" | "--format" => {
                output_format = parse_output_format(&required_arg_at(args, index, arg)?)?;
                index += FLAG_AND_VALUE_ARGS;
            }
            "--locked" => {
                locked = true;
                index += 1;
            }
            "--update-advisory-database" => {
                update_advisory_database = true;
                index += 1;
            }
            "--run" => {
                run = true;
                index += 1;
            }
            _ if arg.starts_with("--rail=") => {
                rail = Some(crate::evidence_rails::EvidenceRailKind::parse(arg.trim_start_matches("--rail="))?);
                index += 1;
            }
            _ if arg.starts_with("--input=") => {
                input = Some(PathBuf::from(arg.trim_start_matches("--input=")));
                index += 1;
            }
            _ if arg.starts_with("--artifact-dir=") => {
                artifact_dir = Some(PathBuf::from(arg.trim_start_matches("--artifact-dir=")));
                index += 1;
            }
            _ if arg.starts_with("--workspace-root=") => {
                workspace_root = Some(PathBuf::from(arg.trim_start_matches("--workspace-root=")));
                index += 1;
            }
            _ if arg.starts_with("--config=") => {
                config = Some(PathBuf::from(arg.trim_start_matches("--config=")));
                index += 1;
            }
            _ if arg.starts_with("--baseline=") => {
                baseline = Some(PathBuf::from(arg.trim_start_matches("--baseline=")));
                index += 1;
            }
            _ if arg.starts_with("--profile=") => {
                profile = Some(arg.trim_start_matches("--profile=").to_owned());
                index += 1;
            }
            _ if arg.starts_with("--command=") => {
                command = Some(arg.trim_start_matches("--command=").to_owned());
                index += 1;
            }
            _ if arg.starts_with("--parent-artifact=") => {
                parent_artifacts.push(arg.trim_start_matches("--parent-artifact=").to_owned());
                index += 1;
            }
            _ if arg.starts_with("--external-report=") => {
                external_reports.push(PathBuf::from(arg.trim_start_matches("--external-report=")));
                index += 1;
            }
            _ if arg.starts_with("--output-format=") => {
                output_format = parse_output_format(arg.trim_start_matches("--output-format="))?;
                index += 1;
            }
            _ if arg.starts_with("--format=") => {
                output_format = parse_output_format(arg.trim_start_matches("--format="))?;
                index += 1;
            }
            _ => return Err(format!("unsupported evidence collect argument `{arg}`")),
        }
    }

    if output_format == OutputFormat::Sarif {
        return Err("evidence collect does not support SARIF output; use human or json".to_owned());
    }

    Ok(EvidenceRailArgs {
        rail: rail.ok_or_else(|| "evidence collect requires `--rail RAIL`".to_owned())?,
        input,
        artifact_dir: artifact_dir.unwrap_or_else(|| PathBuf::from("artifacts/octet-evidence")),
        workspace_root,
        config,
        baseline,
        profile,
        command,
        parent_artifacts,
        external_reports,
        output_format,
        locked,
        update_advisory_database,
        run,
        command_line,
    })
}

fn parse_fingerprint(args: &[String]) -> Result<FingerprintArgs, String> {
    let mut mode = None;
    let mut paths = Vec::new();
    let mut output_format = OutputFormat::Human;
    let mut review_manifest = None;
    let mut index = 0;

    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "--check" => {
                mode = Some(FingerprintMode::Check);
                index += 1;
            }
            "--write" => {
                mode = Some(FingerprintMode::Write);
                index += 1;
            }
            "--update" => {
                mode = Some(FingerprintMode::Update);
                index += 1;
            }
            "--diff" => {
                mode = Some(FingerprintMode::Diff);
                index += 1;
            }
            "--output-format" => {
                let value = args.get(index + 1).ok_or_else(|| "missing value after `--output-format`".to_owned())?;
                output_format = parse_output_format(value)?;
                index += 2;
            }
            _ if arg.starts_with("--output-format=") => {
                output_format = parse_output_format(arg.trim_start_matches("--output-format="))?;
                index += 1;
            }
            "--review-manifest" => {
                let value = args.get(index + 1).ok_or_else(|| "missing value after `--review-manifest`".to_owned())?;
                review_manifest = Some(PathBuf::from(value));
                index += 2;
            }
            _ if arg.starts_with("--review-manifest=") => {
                review_manifest = Some(PathBuf::from(arg.trim_start_matches("--review-manifest=")));
                index += 1;
            }
            _ if arg.starts_with('-') => return Err(format!("unsupported fingerprint argument `{arg}`")),
            _ => {
                paths.push(PathBuf::from(arg));
                index += 1;
            }
        }
    }

    Ok(FingerprintArgs {
        mode: mode.unwrap_or(FingerprintMode::Check),
        paths,
        output_format,
        review_manifest,
    })
}

fn parse_migrate(args: &[String]) -> Result<MigrateArgs, String> {
    match args.first().map(String::as_str) {
        Some("start") => parse_legacy_migrate(MigrateMode::Start, &args[1..]),
        Some("status") => parse_legacy_migrate(MigrateMode::Status, &args[1..]),
        Some("apply-safe") => parse_legacy_migrate(MigrateMode::ApplySafe, &args[1..]),
        Some("session") => parse_migrate_session(&args[1..]),
        Some(other) => Err(format!(
            "unknown migrate subcommand `{other}`; expected `start`, `status`, `apply-safe`, or `session`"
        )),
        None => Err("missing migrate subcommand; expected `start`, `status`, `apply-safe`, or `session`".to_owned()),
    }
}

fn parse_legacy_migrate(mode: MigrateMode, args: &[String]) -> Result<MigrateArgs, String> {
    let mut parsed = parse_migrate_common(args)?;
    parsed.mode = mode;
    Ok(parsed)
}

fn parse_migrate_session(args: &[String]) -> Result<MigrateArgs, String> {
    let mode = match args.first().map(String::as_str) {
        Some("plan") => MigrateMode::SessionPlan,
        Some("status") => MigrateMode::SessionStatus,
        Some("verify") => MigrateMode::SessionVerify,
        Some("apply") => MigrateMode::SessionApply,
        Some(other) => {
            return Err(format!(
                "unknown migrate session action `{other}`; expected `plan`, `status`, `verify`, or `apply`"
            ));
        }
        None => {
            return Err("missing migrate session action; expected `plan`, `status`, `verify`, or `apply`".to_owned());
        }
    };
    let mut parsed = parse_migrate_common(&args[1..])?;
    parsed.mode = mode;
    if parsed.session.is_none() {
        return Err("migrate session requires `--session PATH`".to_owned());
    }
    if mode == MigrateMode::SessionApply && parsed.operation_manifest.is_none() {
        return Err("migrate session apply requires `--operation-manifest PATH`".to_owned());
    }
    Ok(parsed)
}

fn parse_migrate_common(args: &[String]) -> Result<MigrateArgs, String> {
    const FLAG_AND_VALUE_ARGS: usize = 2;

    let mut profile = None;
    let mut output_format = OutputFormat::Human;
    let mut manifest = PathBuf::from(".octet/migrations/octet-migration.json");
    let mut session = None;
    let mut operation_manifest = None;
    let mut dry_run = false;
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "--profile" => {
                profile = Some(required_arg_at(args, index, arg)?);
                index += FLAG_AND_VALUE_ARGS;
            }
            "--format" | "--output-format" => {
                output_format = parse_output_format(&required_arg_at(args, index, arg)?)?;
                index += FLAG_AND_VALUE_ARGS;
            }
            "--manifest" => {
                manifest = PathBuf::from(required_arg_at(args, index, arg)?);
                index += FLAG_AND_VALUE_ARGS;
            }
            "--session" => {
                session = Some(PathBuf::from(required_arg_at(args, index, arg)?));
                index += FLAG_AND_VALUE_ARGS;
            }
            "--operation-manifest" => {
                operation_manifest = Some(PathBuf::from(required_arg_at(args, index, arg)?));
                index += FLAG_AND_VALUE_ARGS;
            }
            "--dry-run" => {
                dry_run = true;
                index += 1;
            }
            _ if arg.starts_with("--profile=") => {
                profile = Some(arg.trim_start_matches("--profile=").to_owned());
                index += 1;
            }
            _ if arg.starts_with("--format=") => {
                output_format = parse_output_format(arg.trim_start_matches("--format="))?;
                index += 1;
            }
            _ if arg.starts_with("--output-format=") => {
                output_format = parse_output_format(arg.trim_start_matches("--output-format="))?;
                index += 1;
            }
            _ if arg.starts_with("--manifest=") => {
                manifest = PathBuf::from(arg.trim_start_matches("--manifest="));
                index += 1;
            }
            _ if arg.starts_with("--session=") => {
                session = Some(PathBuf::from(arg.trim_start_matches("--session=")));
                index += 1;
            }
            _ if arg.starts_with("--operation-manifest=") => {
                operation_manifest = Some(PathBuf::from(arg.trim_start_matches("--operation-manifest=")));
                index += 1;
            }
            _ => return Err(format!("unsupported migrate argument `{arg}`")),
        }
    }
    Ok(MigrateArgs {
        mode: MigrateMode::Start,
        profile,
        output_format,
        manifest,
        session,
        operation_manifest,
        dry_run,
    })
}

fn parse_object(args: &[String]) -> Result<ObjectArgs, String> {
    match args.first().map(String::as_str) {
        Some("emit") => parse_object_scan(ObjectMode::Emit, &args[1..]),
        Some("check") => parse_object_scan(ObjectMode::Check, &args[1..]),
        Some("update") => parse_object_scan(ObjectMode::Update, &args[1..]),
        Some("diff") => parse_object_diff(&args[1..]),
        Some("pointer") => parse_object_pointer(&args[1..]),
        Some("bundle") => parse_object_bundle(&args[1..]),
        Some("corpus") => parse_object_corpus(&args[1..]),
        Some("supplement") => parse_object_supplement(&args[1..]),
        Some(other) => Err(format!(
            "unknown object subcommand `{other}`; expected `emit`, `check`, `update`, `diff`, `pointer`, `bundle`, `corpus`, or `supplement`"
        )),
        None => Err(
            "missing object subcommand; expected `emit`, `check`, `update`, `diff`, `pointer`, `bundle`, `corpus`, or `supplement`"
                .to_owned(),
        ),
    }
}

fn parse_object_scan(mode: ObjectMode, args: &[String]) -> Result<ObjectArgs, String> {
    let mut paths = Vec::new();
    let mut function = None;
    let mut output_format = OutputFormat::Human;
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "--function" => {
                function =
                    Some(args.get(index + 1).ok_or_else(|| "missing value after `--function`".to_owned())?.clone());
                index += 2;
            }
            _ if arg.starts_with("--function=") => {
                function = Some(arg.trim_start_matches("--function=").to_owned());
                index += 1;
            }
            "--output-format" => {
                let value = args.get(index + 1).ok_or_else(|| "missing value after `--output-format`".to_owned())?;
                output_format = parse_output_format(value)?;
                index += 2;
            }
            _ if arg.starts_with("--output-format=") => {
                output_format = parse_output_format(arg.trim_start_matches("--output-format="))?;
                index += 1;
            }
            _ if arg.starts_with('-') => return Err(format!("unsupported object argument `{arg}`")),
            _ => {
                paths.push(PathBuf::from(arg));
                index += 1;
            }
        }
    }
    if paths.is_empty() {
        paths.push(PathBuf::from("."));
    }
    Ok(ObjectArgs {
        mode,
        paths,
        function,
        output_format,
    })
}

fn parse_object_diff(args: &[String]) -> Result<ObjectArgs, String> {
    let mut old = None;
    let mut new = None;
    let mut pointer_manifest = None;
    let mut output_format = OutputFormat::Human;
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "--old" => {
                old = Some(PathBuf::from(args.get(index + 1).ok_or_else(|| "missing value after `--old`".to_owned())?));
                index += 2;
            }
            "--new" => {
                new = Some(PathBuf::from(args.get(index + 1).ok_or_else(|| "missing value after `--new`".to_owned())?));
                index += 2;
            }
            "--pointer-manifest" => {
                pointer_manifest = Some(PathBuf::from(
                    args.get(index + 1).ok_or_else(|| "missing value after `--pointer-manifest`".to_owned())?,
                ));
                index += 2;
            }
            "--output-format" => {
                output_format = parse_output_format(
                    args.get(index + 1).ok_or_else(|| "missing value after `--output-format`".to_owned())?,
                )?;
                index += 2;
            }
            _ if arg.starts_with("--old=") => {
                old = Some(PathBuf::from(arg.trim_start_matches("--old=")));
                index += 1;
            }
            _ if arg.starts_with("--new=") => {
                new = Some(PathBuf::from(arg.trim_start_matches("--new=")));
                index += 1;
            }
            _ if arg.starts_with("--pointer-manifest=") => {
                pointer_manifest = Some(PathBuf::from(arg.trim_start_matches("--pointer-manifest=")));
                index += 1;
            }
            _ if arg.starts_with("--output-format=") => {
                output_format = parse_output_format(arg.trim_start_matches("--output-format="))?;
                index += 1;
            }
            _ => return Err(format!("unsupported object diff argument `{arg}`")),
        }
    }
    Ok(ObjectArgs {
        mode: ObjectMode::Diff {
            old: old.ok_or_else(|| "missing `--old`".to_owned())?,
            new: new.ok_or_else(|| "missing `--new`".to_owned())?,
            pointer_manifest,
        },
        paths: Vec::new(),
        function: None,
        output_format,
    })
}

fn parse_object_pointer(args: &[String]) -> Result<ObjectArgs, String> {
    match args.first().map(String::as_str) {
        Some("set") => {
            let manifest = PathBuf::from(required_arg(&args[1..], "--manifest")?);
            let name = required_arg(&args[1..], "--name")?;
            let hash = required_arg(&args[1..], "--hash")?;
            let objects = PathBuf::from(required_arg(&args[1..], "--objects")?);
            let receipt = PathBuf::from(required_arg(&args[1..], "--receipt")?);
            Ok(ObjectArgs {
                mode: ObjectMode::PointerSet {
                    manifest,
                    name,
                    hash,
                    objects,
                    receipt,
                },
                paths: Vec::new(),
                function: None,
                output_format: OutputFormat::Json,
            })
        }
        Some("validate") => Ok(ObjectArgs {
            mode: ObjectMode::PointerValidate {
                manifest: PathBuf::from(required_arg(&args[1..], "--manifest")?),
                objects: PathBuf::from(required_arg(&args[1..], "--objects")?),
            },
            paths: Vec::new(),
            function: None,
            output_format: OutputFormat::Json,
        }),
        Some(other) => Err(format!("unknown object pointer action `{other}`; expected `set` or `validate`")),
        None => Err("missing object pointer action; expected `set` or `validate`".to_owned()),
    }
}

fn parse_object_bundle(args: &[String]) -> Result<ObjectArgs, String> {
    match args.first().map(String::as_str) {
        Some("export") => Ok(ObjectArgs {
            mode: ObjectMode::BundleExport {
                objects: PathBuf::from(required_arg(&args[1..], "--objects")?),
                pointers: optional_arg(&args[1..], "--pointers").map(PathBuf::from),
                output: PathBuf::from(required_arg(&args[1..], "--output")?),
                replay_command: required_arg(&args[1..], "--replay-command")?,
            },
            paths: Vec::new(),
            function: None,
            output_format: OutputFormat::Json,
        }),
        Some(other) => Err(format!("unknown object bundle action `{other}`; expected `export`")),
        None => Err("missing object bundle action; expected `export`".to_owned()),
    }
}

#[allow(clippy::too_many_lines)]
fn parse_object_corpus(args: &[String]) -> Result<ObjectArgs, String> {
    match args.first().map(String::as_str) {
        Some("receipt") => {
            let mut paths = Vec::new();
            let mut output = None;
            let mut index = 1;
            while index < args.len() {
                let arg = &args[index];
                if arg == "--output" {
                    output = Some(PathBuf::from(
                        args.get(index + 1).ok_or_else(|| "missing value after `--output`".to_owned())?,
                    ));
                    index += 2;
                } else if let Some(value) = arg.strip_prefix("--output=") {
                    output = Some(PathBuf::from(value));
                    index += 1;
                } else if arg.starts_with('-') {
                    return Err(format!("unsupported object corpus receipt argument `{arg}`"));
                } else {
                    paths.push(PathBuf::from(arg));
                    index += 1;
                }
            }
            if paths.is_empty() {
                paths.push(PathBuf::from("."));
            }
            Ok(ObjectArgs {
                mode: ObjectMode::CorpusReceipt {
                    output: output.ok_or_else(|| "missing `--output`".to_owned())?,
                },
                paths,
                function: None,
                output_format: OutputFormat::Json,
            })
        }
        Some("check") => {
            let mut paths = Vec::new();
            let mut receipt = None;
            let mut output_format = OutputFormat::Human;
            let mut index = 1;
            while index < args.len() {
                let arg = &args[index];
                if arg == "--receipt" {
                    receipt = Some(PathBuf::from(
                        args.get(index + 1).ok_or_else(|| "missing value after `--receipt`".to_owned())?,
                    ));
                    index += 2;
                } else if let Some(value) = arg.strip_prefix("--receipt=") {
                    receipt = Some(PathBuf::from(value));
                    index += 1;
                } else if arg == "--output-format" {
                    output_format = parse_output_format(
                        args.get(index + 1).ok_or_else(|| "missing value after `--output-format`".to_owned())?,
                    )?;
                    index += 2;
                } else if let Some(value) = arg.strip_prefix("--output-format=") {
                    output_format = parse_output_format(value)?;
                    index += 1;
                } else if arg.starts_with('-') {
                    return Err(format!("unsupported object corpus check argument `{arg}`"));
                } else {
                    paths.push(PathBuf::from(arg));
                    index += 1;
                }
            }
            if paths.is_empty() {
                paths.push(PathBuf::from("."));
            }
            Ok(ObjectArgs {
                mode: ObjectMode::CorpusCheck {
                    receipt: receipt.ok_or_else(|| "missing `--receipt`".to_owned())?,
                },
                paths,
                function: None,
                output_format,
            })
        }
        Some("caveats") => {
            let mut paths = Vec::new();
            let mut output_format = OutputFormat::Human;
            let mut index = 1;
            while index < args.len() {
                let arg = &args[index];
                if arg == "--output-format" {
                    output_format = parse_output_format(
                        args.get(index + 1).ok_or_else(|| "missing value after `--output-format`".to_owned())?,
                    )?;
                    index += 2;
                } else if let Some(value) = arg.strip_prefix("--output-format=") {
                    output_format = parse_output_format(value)?;
                    index += 1;
                } else if arg.starts_with('-') {
                    return Err(format!("unsupported object corpus caveats argument `{arg}`"));
                } else {
                    paths.push(PathBuf::from(arg));
                    index += 1;
                }
            }
            if paths.is_empty() {
                paths.push(PathBuf::from("."));
            }
            Ok(ObjectArgs {
                mode: ObjectMode::CorpusCaveats,
                paths,
                function: None,
                output_format,
            })
        }
        Some("contract") => {
            let mut output_format = OutputFormat::Human;
            let mut index = 1;
            while index < args.len() {
                let arg = &args[index];
                if arg == "--output-format" {
                    output_format = parse_output_format(
                        args.get(index + 1).ok_or_else(|| "missing value after `--output-format`".to_owned())?,
                    )?;
                    index += 2;
                } else if let Some(value) = arg.strip_prefix("--output-format=") {
                    output_format = parse_output_format(value)?;
                    index += 1;
                } else {
                    return Err(format!("unsupported object corpus contract argument `{arg}`"));
                }
            }
            Ok(ObjectArgs {
                mode: ObjectMode::CorpusContract,
                paths: Vec::new(),
                function: None,
                output_format,
            })
        }
        Some(other) => Err(format!(
            "unknown object corpus action `{other}`; expected `receipt`, `check`, `caveats`, or `contract`"
        )),
        None => Err("missing object corpus action; expected `receipt`, `check`, `caveats`, or `contract`".to_owned()),
    }
}

fn parse_object_supplement(args: &[String]) -> Result<ObjectArgs, String> {
    match args.first().map(String::as_str) {
        Some("fixtures") => parse_object_supplement_fixtures(&args[1..]),
        Some("emit") => parse_object_supplement_scan(true, &args[1..]),
        Some("check") => parse_object_supplement_scan(false, &args[1..]),
        Some(other) => {
            Err(format!("unknown object supplement action `{other}`; expected `fixtures`, `emit`, or `check`"))
        }
        None => Err("missing object supplement action; expected `fixtures`, `emit`, or `check`".to_owned()),
    }
}

fn parse_object_supplement_fixtures(args: &[String]) -> Result<ObjectArgs, String> {
    let mut family = None;
    let mut scope = None;
    let mut output_format = OutputFormat::Human;
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "--family" => {
                family = Some(parse_supplement_family(
                    args.get(index + 1).ok_or_else(|| "missing value after `--family`".to_owned())?,
                )?);
                index += 2;
            }
            "--spike" => {
                scope = Some(parse_supplement_spike_scope(
                    args.get(index + 1).ok_or_else(|| "missing value after `--spike`".to_owned())?,
                )?);
                index += 2;
            }
            "--output-format" => {
                output_format = parse_output_format(
                    args.get(index + 1).ok_or_else(|| "missing value after `--output-format`".to_owned())?,
                )?;
                index += 2;
            }
            _ if arg.starts_with("--family=") => {
                family = Some(parse_supplement_family(arg.trim_start_matches("--family="))?);
                index += 1;
            }
            _ if arg.starts_with("--spike=") => {
                scope = Some(parse_supplement_spike_scope(arg.trim_start_matches("--spike="))?);
                index += 1;
            }
            _ if arg.starts_with("--output-format=") => {
                output_format = parse_output_format(arg.trim_start_matches("--output-format="))?;
                index += 1;
            }
            _ if arg.starts_with('-') => {
                return Err(format!("unsupported object supplement fixtures argument `{arg}`"));
            }
            _ => return Err(format!("unexpected object supplement fixtures path `{arg}`")),
        }
    }
    let family = family.ok_or_else(|| {
        "object supplement fixtures requires `--family hir`, `--family thir`, or `--family mir`".to_owned()
    })?;
    let scope = scope.ok_or_else(|| {
        "object supplement fixtures requires `--spike rustc-hir-fixture`, `--spike rustc-thir-fixture`, or `--spike rustc-mir-fixture`".to_owned()
    })?;
    match (family, scope) {
        (SupplementFamilyArg::Hir, SupplementScopeArg::RustcHirFixtureSpike)
        | (SupplementFamilyArg::Thir, SupplementScopeArg::RustcThirFixtureSpike)
        | (SupplementFamilyArg::Mir, SupplementScopeArg::RustcMirFixtureSpike) => {}
        (SupplementFamilyArg::Hir, _) => {
            return Err("object supplement fixtures for `--family hir` requires `--spike rustc-hir-fixture`".to_owned());
        }
        (SupplementFamilyArg::Thir, _) => {
            return Err(
                "object supplement fixtures for `--family thir` requires `--spike rustc-thir-fixture`".to_owned()
            );
        }
        (SupplementFamilyArg::Mir, _) => {
            return Err("object supplement fixtures for `--family mir` requires `--spike rustc-mir-fixture`".to_owned());
        }
    }
    Ok(ObjectArgs {
        mode: ObjectMode::SupplementFixtures { family, scope },
        paths: Vec::new(),
        function: None,
        output_format,
    })
}

#[allow(clippy::too_many_lines)]
fn parse_object_supplement_scan(emit: bool, args: &[String]) -> Result<ObjectArgs, String> {
    let mut paths = Vec::new();
    let mut function = None;
    let mut family = None;
    let mut scope = SupplementScopeArg::Pilot;
    let mut receipt = None;
    let mut output_format = OutputFormat::Human;
    let mut fixture_shape = None;
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "--family" => {
                family = Some(parse_supplement_family(
                    args.get(index + 1).ok_or_else(|| "missing value after `--family`".to_owned())?,
                )?);
                index += 2;
            }
            "--function" => {
                function =
                    Some(args.get(index + 1).ok_or_else(|| "missing value after `--function`".to_owned())?.clone());
                index += 2;
            }
            "--receipt" => {
                receipt = Some(PathBuf::from(
                    args.get(index + 1).ok_or_else(|| "missing value after `--receipt`".to_owned())?,
                ));
                index += 2;
            }
            "--spike" => {
                scope = parse_supplement_spike_scope(
                    args.get(index + 1).ok_or_else(|| "missing value after `--spike`".to_owned())?,
                )?;
                index += 2;
            }
            "--fixture-shape" => {
                fixture_shape = Some(
                    args.get(index + 1).ok_or_else(|| "missing value after `--fixture-shape`".to_owned())?.clone(),
                );
                index += 2;
            }
            "--output-format" => {
                output_format = parse_output_format(
                    args.get(index + 1).ok_or_else(|| "missing value after `--output-format`".to_owned())?,
                )?;
                index += 2;
            }
            _ if arg.starts_with("--family=") => {
                family = Some(parse_supplement_family(arg.trim_start_matches("--family="))?);
                index += 1;
            }
            _ if arg.starts_with("--function=") => {
                function = Some(arg.trim_start_matches("--function=").to_owned());
                index += 1;
            }
            _ if arg.starts_with("--receipt=") => {
                receipt = Some(PathBuf::from(arg.trim_start_matches("--receipt=")));
                index += 1;
            }
            _ if arg.starts_with("--spike=") => {
                scope = parse_supplement_spike_scope(arg.trim_start_matches("--spike="))?;
                index += 1;
            }
            _ if arg.starts_with("--fixture-shape=") => {
                fixture_shape = Some(arg.trim_start_matches("--fixture-shape=").to_owned());
                index += 1;
            }
            _ if arg.starts_with("--output-format=") => {
                output_format = parse_output_format(arg.trim_start_matches("--output-format="))?;
                index += 1;
            }
            _ if arg.starts_with('-') => return Err(format!("unsupported object supplement argument `{arg}`")),
            _ => {
                paths.push(PathBuf::from(arg));
                index += 1;
            }
        }
    }
    let fixture_shape_was_provided = fixture_shape.is_some();
    if let Some(shape) = fixture_shape {
        if !matches!(
            scope,
            SupplementScopeArg::RustcHirFixtureSpike
                | SupplementScopeArg::RustcThirFixtureSpike
                | SupplementScopeArg::RustcMirFixtureSpike
        ) {
            return Err(
                "`--fixture-shape` requires `--spike rustc-hir-fixture`, `--spike rustc-thir-fixture`, or `--spike rustc-mir-fixture`".to_owned()
            );
        }
        if !paths.is_empty() || function.is_some() {
            return Err("`--fixture-shape` cannot be combined with explicit fixture paths or `--function`".to_owned());
        }
        if matches!(scope, SupplementScopeArg::RustcMirFixtureSpike)
            && !matches!(shape.trim(), "checked-arithmetic" | "call-return" | "branch-control-flow")
        {
            return Err(
                "`--spike rustc-mir-fixture` currently supports only `--fixture-shape checked-arithmetic`, `--fixture-shape call-return`, or `--fixture-shape branch-control-flow`".to_owned()
            );
        }
        let selected = parse_supplement_fixture_shape(scope, &shape)?;
        paths.push(selected.path);
        function = Some(selected.function);
    }
    let family = family.ok_or_else(|| "missing `--family`".to_owned())?;
    match (family, scope) {
        (SupplementFamilyArg::Hir | SupplementFamilyArg::Mir, SupplementScopeArg::RustcThirFixtureSpike) => {
            return Err("rustc THIR fixture spike only supports `--family thir`".to_owned());
        }
        (SupplementFamilyArg::Thir | SupplementFamilyArg::Mir, SupplementScopeArg::RustcHirFixtureSpike) => {
            return Err("rustc HIR fixture spike only supports `--family hir`".to_owned());
        }
        (SupplementFamilyArg::Hir | SupplementFamilyArg::Thir, SupplementScopeArg::RustcMirFixtureSpike) => {
            return Err("rustc MIR fixture spike only supports `--family mir`".to_owned());
        }
        _ => {}
    }
    if matches!((family, scope), (SupplementFamilyArg::Thir, SupplementScopeArg::RustcThirFixtureSpike))
        && !fixture_shape_was_provided
    {
        return Err(
            "`--family thir --spike rustc-thir-fixture` requires explicit `--fixture-shape checked-arithmetic`, `--fixture-shape call-return`, or `--fixture-shape branch-control-flow`"
                .to_owned(),
        );
    }
    if matches!((family, scope), (SupplementFamilyArg::Mir, SupplementScopeArg::RustcMirFixtureSpike))
        && !fixture_shape_was_provided
    {
        return Err("`--family mir --spike rustc-mir-fixture` requires explicit `--fixture-shape checked-arithmetic`, `--fixture-shape call-return`, or `--fixture-shape branch-control-flow`"
            .to_owned());
    }
    if paths.is_empty() {
        paths.push(PathBuf::from("."));
    }
    let mode = if emit {
        ObjectMode::SupplementEmit { family, scope }
    } else {
        ObjectMode::SupplementCheck {
            family,
            scope,
            receipt: receipt.ok_or_else(|| "missing `--receipt`".to_owned())?,
        }
    };
    Ok(ObjectArgs {
        mode,
        paths,
        function,
        output_format,
    })
}

fn parse_supplement_family(value: &str) -> Result<SupplementFamilyArg, String> {
    match value {
        "hir" => Ok(SupplementFamilyArg::Hir),
        "thir" => Ok(SupplementFamilyArg::Thir),
        "mir" => Ok(SupplementFamilyArg::Mir),
        other => Err(format!("unsupported supplement family `{other}`; expected `hir`, `thir`, or `mir`")),
    }
}

fn parse_supplement_spike_scope(value: &str) -> Result<SupplementScopeArg, String> {
    match value {
        "rustc-hir-fixture" | "rustc-hir-fixture-spike" => Ok(SupplementScopeArg::RustcHirFixtureSpike),
        "rustc-thir-fixture" | "rustc-thir-fixture-spike" => Ok(SupplementScopeArg::RustcThirFixtureSpike),
        "rustc-mir-fixture" | "rustc-mir-fixture-spike" => Ok(SupplementScopeArg::RustcMirFixtureSpike),
        other => Err(format!(
            "unsupported supplement spike `{other}`; expected `rustc-hir-fixture`, `rustc-thir-fixture`, or `rustc-mir-fixture`"
        )),
    }
}

fn parse_supplement_fixture_shape(
    scope: SupplementScopeArg,
    value: &str,
) -> Result<crate::object_core::SupplementFixtureSelection, String> {
    crate::object_core::parse_supplement_fixture_shape(scope, value)
}

fn optional_arg(args: &[String], flag: &str) -> Option<String> {
    if let Some(value) = args.iter().find_map(|arg| arg.strip_prefix(&format!("{flag}="))) {
        return Some(value.to_owned());
    }
    let index = args.iter().position(|arg| arg == flag)?;
    args.get(index + 1).cloned()
}

fn required_arg(args: &[String], flag: &str) -> Result<String, String> {
    if let Some(value) = args.iter().find_map(|arg| arg.strip_prefix(&format!("{flag}="))) {
        return Ok(value.to_owned());
    }
    let index = args.iter().position(|arg| arg == flag).ok_or_else(|| format!("missing `{flag}`"))?;
    args.get(index + 1).cloned().ok_or_else(|| format!("missing value after `{flag}`"))
}

fn required_arg_at(args: &[String], index: usize, flag: &str) -> Result<String, String> {
    let value_index = index.checked_add(1).ok_or_else(|| format!("argument index overflow while parsing `{flag}`"))?;
    args.get(value_index).cloned().ok_or_else(|| format!("missing value after `{flag}`"))
}

fn parse_prove(args: &[String], command_line: String) -> Result<ProveArgs, String> {
    let mut registry = None;
    let mut policy_artifacts_dir = None;
    let mut source = Vec::new();
    let mut family = Vec::new();
    let mut rail = Vec::new();
    let mut artifact_dir = None;
    let mut output_format = OutputFormat::Human;
    let mut list = false;
    let mut explain = false;
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "--registry" => {
                registry = Some(PathBuf::from(required_arg_at(args, index, arg)?));
                index += 2;
            }
            "--policy-artifacts-dir" => {
                policy_artifacts_dir = Some(PathBuf::from(required_arg_at(args, index, arg)?));
                index += 2;
            }
            "--source" => {
                source.push(PathBuf::from(required_arg_at(args, index, arg)?));
                index += 2;
            }
            "--family" => {
                family.push(required_arg_at(args, index, arg)?);
                index += 2;
            }
            "--rail" => {
                rail.push(required_arg_at(args, index, arg)?);
                index += 2;
            }
            "--artifact-dir" => {
                artifact_dir = Some(PathBuf::from(required_arg_at(args, index, arg)?));
                index += 2;
            }
            "--output-format" | "--format" => {
                output_format = parse_output_format(&required_arg_at(args, index, arg)?)?;
                index += 2;
            }
            "--list" => {
                list = true;
                index += 1;
            }
            "--explain" => {
                explain = true;
                index += 1;
            }
            _ if arg.starts_with("--registry=") => {
                registry = Some(PathBuf::from(arg.trim_start_matches("--registry=")));
                index += 1;
            }
            _ if arg.starts_with("--policy-artifacts-dir=") => {
                policy_artifacts_dir = Some(PathBuf::from(arg.trim_start_matches("--policy-artifacts-dir=")));
                index += 1;
            }
            _ if arg.starts_with("--source=") => {
                source.push(PathBuf::from(arg.trim_start_matches("--source=")));
                index += 1;
            }
            _ if arg.starts_with("--family=") => {
                family.push(arg.trim_start_matches("--family=").to_owned());
                index += 1;
            }
            _ if arg.starts_with("--rail=") => {
                rail.push(arg.trim_start_matches("--rail=").to_owned());
                index += 1;
            }
            _ if arg.starts_with("--artifact-dir=") => {
                artifact_dir = Some(PathBuf::from(arg.trim_start_matches("--artifact-dir=")));
                index += 1;
            }
            _ if arg.starts_with("--output-format=") => {
                output_format = parse_output_format(arg.trim_start_matches("--output-format="))?;
                index += 1;
            }
            _ if arg.starts_with("--format=") => {
                output_format = parse_output_format(arg.trim_start_matches("--format="))?;
                index += 1;
            }
            _ => return Err(format!("unsupported prove argument `{arg}`")),
        }
    }

    if registry.is_some() == policy_artifacts_dir.is_some() {
        return Err("prove requires exactly one of `--registry PATH` or `--policy-artifacts-dir PATH`".to_owned());
    }
    if source.is_empty() {
        return Err("prove requires at least one `--source PATH`".to_owned());
    }
    if output_format == OutputFormat::Sarif {
        return Err("prove does not support SARIF output; use human or json".to_owned());
    }

    Ok(ProveArgs {
        registry,
        policy_artifacts_dir,
        source,
        family,
        rail,
        artifact_dir: artifact_dir.unwrap_or_else(|| PathBuf::from("artifacts/octet")),
        output_format,
        command_line,
        list,
        explain,
    })
}

fn parse_proof_ref(args: &[String]) -> Result<ProofRefArgs, String> {
    match args.first().map(String::as_str) {
        Some("validate") => parse_proof_ref_validate(&args[1..]),
        Some("scaffold") => parse_proof_ref_scaffold(&args[1..]),
        Some(other) => Err(format!("unsupported proof-ref subcommand `{other}`; expected `validate` or `scaffold`")),
        None => Err("missing proof-ref subcommand; expected `validate` or `scaffold`".to_owned()),
    }
}

fn parse_proof_ref_validate(args: &[String]) -> Result<ProofRefArgs, String> {
    let mut proof_cards = None;
    let mut reference = None;
    let mut output_format = OutputFormat::Human;
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "--proof-cards" => {
                proof_cards = Some(PathBuf::from(required_arg_at(args, index, arg)?));
                index += 2;
            }
            "--ref" => {
                reference = Some(PathBuf::from(required_arg_at(args, index, arg)?));
                index += 2;
            }
            "--output-format" | "--format" => {
                output_format = parse_output_format(&required_arg_at(args, index, arg)?)?;
                index += 2;
            }
            _ if arg.starts_with("--proof-cards=") => {
                proof_cards = Some(PathBuf::from(arg.trim_start_matches("--proof-cards=")));
                index += 1;
            }
            _ if arg.starts_with("--ref=") => {
                reference = Some(PathBuf::from(arg.trim_start_matches("--ref=")));
                index += 1;
            }
            _ if arg.starts_with("--output-format=") => {
                output_format = parse_output_format(arg.trim_start_matches("--output-format="))?;
                index += 1;
            }
            _ if arg.starts_with("--format=") => {
                output_format = parse_output_format(arg.trim_start_matches("--format="))?;
                index += 1;
            }
            _ => return Err(format!("unsupported proof-ref validate argument `{arg}`")),
        }
    }
    if output_format == OutputFormat::Sarif {
        return Err("proof-ref does not support SARIF output; use human or json".to_owned());
    }
    Ok(ProofRefArgs {
        mode: ProofRefMode::Validate(ProofRefValidateArgs {
            proof_cards: proof_cards.ok_or_else(|| "proof-ref validate requires `--proof-cards PATH`".to_owned())?,
            reference: reference.ok_or_else(|| "proof-ref validate requires `--ref PATH`".to_owned())?,
            output_format,
        }),
    })
}

fn parse_proof_ref_scaffold(args: &[String]) -> Result<ProofRefArgs, String> {
    let mut proof_cards = None;
    let mut proof_or_property_id = None;
    let mut reference_id = None;
    let mut consumer_context = None;
    let mut consumer_source = None;
    let mut anchor = None;
    let mut output = None;
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "--proof-cards" => {
                proof_cards = Some(PathBuf::from(required_arg_at(args, index, arg)?));
                index += 2;
            }
            "--proof-or-property-id" => {
                proof_or_property_id = Some(required_arg_at(args, index, arg)?);
                index += 2;
            }
            "--reference-id" => {
                reference_id = Some(required_arg_at(args, index, arg)?);
                index += 2;
            }
            "--consumer-context" => {
                consumer_context = Some(required_arg_at(args, index, arg)?);
                index += 2;
            }
            "--consumer-source" => {
                consumer_source = Some(PathBuf::from(required_arg_at(args, index, arg)?));
                index += 2;
            }
            "--anchor" => {
                anchor = Some(required_arg_at(args, index, arg)?);
                index += 2;
            }
            "--output" => {
                output = Some(PathBuf::from(required_arg_at(args, index, arg)?));
                index += 2;
            }
            _ if arg.starts_with("--proof-cards=") => {
                proof_cards = Some(PathBuf::from(arg.trim_start_matches("--proof-cards=")));
                index += 1;
            }
            _ if arg.starts_with("--proof-or-property-id=") => {
                proof_or_property_id = Some(arg.trim_start_matches("--proof-or-property-id=").to_owned());
                index += 1;
            }
            _ if arg.starts_with("--reference-id=") => {
                reference_id = Some(arg.trim_start_matches("--reference-id=").to_owned());
                index += 1;
            }
            _ if arg.starts_with("--consumer-context=") => {
                consumer_context = Some(arg.trim_start_matches("--consumer-context=").to_owned());
                index += 1;
            }
            _ if arg.starts_with("--consumer-source=") => {
                consumer_source = Some(PathBuf::from(arg.trim_start_matches("--consumer-source=")));
                index += 1;
            }
            _ if arg.starts_with("--anchor=") => {
                anchor = Some(arg.trim_start_matches("--anchor=").to_owned());
                index += 1;
            }
            _ if arg.starts_with("--output=") => {
                output = Some(PathBuf::from(arg.trim_start_matches("--output=")));
                index += 1;
            }
            _ => return Err(format!("unsupported proof-ref scaffold argument `{arg}`")),
        }
    }
    Ok(ProofRefArgs {
        mode: ProofRefMode::Scaffold(ProofRefScaffoldArgs {
            proof_cards: proof_cards.ok_or_else(|| "proof-ref scaffold requires `--proof-cards PATH`".to_owned())?,
            proof_or_property_id: proof_or_property_id
                .ok_or_else(|| "proof-ref scaffold requires `--proof-or-property-id ID`".to_owned())?,
            reference_id: reference_id.ok_or_else(|| "proof-ref scaffold requires `--reference-id ID`".to_owned())?,
            consumer_context: consumer_context
                .ok_or_else(|| "proof-ref scaffold requires `--consumer-context TEXT`".to_owned())?,
            consumer_source: consumer_source
                .ok_or_else(|| "proof-ref scaffold requires `--consumer-source PATH`".to_owned())?,
            anchor: anchor.ok_or_else(|| "proof-ref scaffold requires `--anchor TEXT`".to_owned())?,
            output,
        }),
    })
}

fn parse_provider_manifest(args: &[String]) -> Result<ProviderManifestArgs, String> {
    match args.first().map(String::as_str) {
        Some("export") => parse_provider_manifest_export(&args[1..]),
        Some("validate") => parse_provider_manifest_validate(&args[1..]),
        Some(other) => Err(format!("unsupported provider-manifest subcommand `{other}`")),
        None => Err("missing provider-manifest subcommand".to_owned()),
    }
}

fn parse_provider_manifest_export(args: &[String]) -> Result<ProviderManifestArgs, String> {
    let mut driver = None;
    let mut lint_library = None;
    let mut config = None;
    let mut standards_policy = None;
    let mut rustc = None;
    let mut profile = None;
    let mut required_lints = Vec::new();
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "--driver" => {
                driver = Some(PathBuf::from(required_arg_at(args, index, "--driver")?));
                index += 2;
            }
            "--lint-library" => {
                lint_library = Some(PathBuf::from(required_arg_at(args, index, "--lint-library")?));
                index += 2;
            }
            "--config" => {
                config = Some(PathBuf::from(required_arg_at(args, index, "--config")?));
                index += 2;
            }
            "--standards-policy" => {
                standards_policy = Some(PathBuf::from(required_arg_at(args, index, "--standards-policy")?));
                index += 2;
            }
            "--rustc" => {
                rustc = Some(PathBuf::from(required_arg_at(args, index, "--rustc")?));
                index += 2;
            }
            "--profile" => {
                profile = Some(required_arg_at(args, index, "--profile")?);
                index += 2;
            }
            "--required-lint" => {
                required_lints.push(required_arg_at(args, index, "--required-lint")?);
                index += 2;
            }
            _ if arg.starts_with("--driver=") => {
                driver = Some(PathBuf::from(arg.trim_start_matches("--driver=")));
                index += 1;
            }
            _ if arg.starts_with("--lint-library=") => {
                lint_library = Some(PathBuf::from(arg.trim_start_matches("--lint-library=")));
                index += 1;
            }
            _ if arg.starts_with("--config=") => {
                config = Some(PathBuf::from(arg.trim_start_matches("--config=")));
                index += 1;
            }
            _ if arg.starts_with("--standards-policy=") => {
                standards_policy = Some(PathBuf::from(arg.trim_start_matches("--standards-policy=")));
                index += 1;
            }
            _ if arg.starts_with("--rustc=") => {
                rustc = Some(PathBuf::from(arg.trim_start_matches("--rustc=")));
                index += 1;
            }
            _ if arg.starts_with("--profile=") => {
                profile = Some(arg.trim_start_matches("--profile=").to_owned());
                index += 1;
            }
            _ if arg.starts_with("--required-lint=") => {
                required_lints.push(arg.trim_start_matches("--required-lint=").to_owned());
                index += 1;
            }
            _ => return Err(format!("unsupported provider-manifest export argument `{arg}`")),
        }
    }
    Ok(ProviderManifestArgs {
        mode: ProviderManifestMode::Export {
            driver: driver.ok_or_else(|| "missing `--driver`".to_owned())?,
            lint_library: lint_library.ok_or_else(|| "missing `--lint-library`".to_owned())?,
            config: config.ok_or_else(|| "missing `--config`".to_owned())?,
            standards_policy,
            rustc,
            profile,
            required_lints,
        },
    })
}

fn parse_provider_manifest_validate(args: &[String]) -> Result<ProviderManifestArgs, String> {
    let mut manifest = None;
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if arg == "--manifest" {
            manifest = Some(PathBuf::from(required_arg_at(args, index, "--manifest")?));
            index += 2;
        } else if let Some(value) = arg.strip_prefix("--manifest=") {
            manifest = Some(PathBuf::from(value));
            index += 1;
        } else {
            return Err(format!("unsupported provider-manifest validate argument `{arg}`"));
        }
    }
    Ok(ProviderManifestArgs {
        mode: ProviderManifestMode::Validate {
            manifest: manifest.ok_or_else(|| "missing `--manifest`".to_owned())?,
        },
    })
}

fn parse_transcript(args: &[String]) -> Result<TranscriptArgs, String> {
    let mut path = None;
    let mut output_format = OutputFormat::Human;
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if arg == "--output-format" || arg == "--format" {
            output_format =
                parse_output_format(args.get(index + 1).ok_or_else(|| format!("missing value after `{arg}`"))?)?;
            index += 2;
        } else if let Some(value) = arg.strip_prefix("--output-format=").or_else(|| arg.strip_prefix("--format=")) {
            output_format = parse_output_format(value)?;
            index += 1;
        } else if arg.starts_with('-') {
            return Err(format!("unsupported transcript argument `{arg}`"));
        } else if path.is_none() {
            path = Some(PathBuf::from(arg));
            index += 1;
        } else {
            return Err(format!("unexpected extra transcript path `{arg}`"));
        }
    }
    Ok(TranscriptArgs {
        path: path.ok_or_else(|| "missing transcript path".to_owned())?,
        output_format,
    })
}

fn parse_output_format(value: &str) -> Result<OutputFormat, String> {
    match value {
        "human" => Ok(OutputFormat::Human),
        "json" => Ok(OutputFormat::Json),
        "sarif" => Ok(OutputFormat::Sarif),
        other => Err(format!("unsupported output format `{other}`; expected `human`, `json`, or `sarif`")),
    }
}

fn format_command_line(program: &str, args: &[String]) -> String {
    std::iter::once(program)
        .chain(args.iter().map(String::as_str))
        .map(shell_quote)
        .collect::<Vec<_>>()
        .join(" ")
}

fn shell_quote(argument: &str) -> String {
    if !argument.is_empty() && argument.chars().all(is_shell_safe) {
        return argument.to_owned();
    }

    format!("'{}'", argument.replace('\'', "'\"'\"'"))
}

fn is_shell_safe(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.' | '/' | ':' | '=' | '@')
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::Path;

    use super::AdmissionMode;
    use super::CommandArgs;
    use super::EvidenceIndexMode;
    use super::FingerprintMode;
    use super::HelpTopic;
    use super::MigrateMode;
    use super::OutputFormat;
    use super::ProofRefMode;
    use super::format_command_line;
    use super::parse;

    const EXPECTED_BINARY_DECLARATION_COUNT: usize = 1;

    #[test]
    fn octet_identity_surfaces() {
        let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        let manifest = std::fs::read_to_string(&manifest_path)
            .unwrap_or_else(|error| panic!("read {}: {error}", manifest_path.display()));

        assert!(manifest.contains("[package]\nname = \"cargo-octet\""));
        assert_eq!(
            manifest.matches("[[bin]]\nname = \"cargo-octet\"\npath = \"src/main.rs\"").count(),
            EXPECTED_BINARY_DECLARATION_COUNT,
            "canonical cargo-octet binary must be declared exactly once"
        );
        assert_eq!(
            manifest.matches("[[bin]]\nname = \"cargo-slotcar\"\npath = \"src/bin/cargo-slotcar.rs\"").count(),
            EXPECTED_BINARY_DECLARATION_COUNT,
            "legacy cargo-slotcar binary remains one explicit compatibility alias"
        );
        assert_eq!(
            manifest
                .matches("[[bin]]\nname = \"cargo-tigerstyle\"\npath = \"src/bin/cargo-tigerstyle.rs\"")
                .count(),
            EXPECTED_BINARY_DECLARATION_COUNT,
            "Tiger Style binary remains one explicit compatibility alias"
        );
        assert!(!manifest.contains("[package]\nname = \"cargo-slotcar\""));

        let canonical = parse(["cargo", "octet", "check", "--output-format", "json"].map(OsString::from))
            .expect("parse canonical check args");
        let alias = parse(["cargo", "slotcar", "check", "--output-format", "json"].map(OsString::from))
            .expect("parse compatibility check args");
        match (canonical, alias) {
            (CommandArgs::Check(canonical_args), CommandArgs::Check(alias_args)) => {
                assert_eq!(canonical_args.command_line, "cargo octet check --output-format json");
                assert_eq!(alias_args.command_line, canonical_args.command_line);
            }
            other => panic!("expected check args, got {other:?}"),
        }
    }

    #[test]
    fn octet_config_and_command_paths() {
        let parsed = parse(["cargo", "octet", "check", "--", "--all-targets"].map(OsString::from))
            .expect("parse canonical check command");
        match parsed {
            CommandArgs::Check(args) => {
                assert_eq!(args.command_line, "cargo octet check -- --all-targets");
                assert_eq!(args.cargo_check_args.as_deref(), Some(["--all-targets".to_owned()].as_slice()));
            }
            other => panic!("expected check args, got {other:?}"),
        }

        let error = parse(["cargo", "octet", "unknown"].map(OsString::from)).expect_err("unknown command should fail");
        assert!(error.contains("unknown subcommand"));
    }

    #[test]
    fn check_parse_accepts_one_strict_profile_selection() {
        let parsed = parse(
            [
                "cargo",
                "octet",
                "check",
                "--profile=deterministic-core",
                "-p",
                "domain",
            ]
            .map(OsString::from),
        )
        .expect("parse strict profile selection");

        match parsed {
            CommandArgs::Check(args) => {
                assert_eq!(args.profile.as_deref(), Some("deterministic-core"));
                assert_eq!(args.scope_args, ["-p", "domain"]);
            }
            other => panic!("expected check args, got {other:?}"),
        }

        let missing = parse(["cargo", "octet", "check", "--profile"].map(OsString::from))
            .expect_err("missing profile value fails");
        let duplicate = parse(
            [
                "cargo",
                "octet",
                "check",
                "--profile",
                "robust",
                "--profile=bounded-systems",
            ]
            .map(OsString::from),
        )
        .expect_err("duplicate profile selection fails");
        assert!(missing.contains("missing value after `--profile`"));
        assert!(duplicate.contains("may be provided only once"));
    }

    // r[verify octet.effect_closure.cli]
    #[test]
    fn admission_parse_requires_explicit_profile_and_closure() {
        let parsed = parse(
            [
                "cargo",
                "octet",
                "admit",
                "--profile=deterministic-core",
                "--closure",
                "--policy",
                "policy.json",
                "--artifact-dir=artifacts/admission",
                "-p",
                "domain",
                "--",
                "--lib",
            ]
            .map(OsString::from),
        )
        .expect("parse closure admission");

        match parsed {
            CommandArgs::Admit(args) => {
                assert_eq!(args.artifact_dir, std::path::PathBuf::from("artifacts/admission"));
                match args.mode {
                    AdmissionMode::Run {
                        profile,
                        closure,
                        policy,
                        scope_args,
                        cargo_check_args,
                    } => {
                        assert_eq!(profile, "deterministic-core");
                        assert!(closure);
                        assert_eq!(policy, std::path::PathBuf::from("policy.json"));
                        assert_eq!(scope_args, ["-p", "domain"]);
                        assert_eq!(cargo_check_args.as_deref(), Some(["--lib".to_owned()].as_slice()));
                    }
                    AdmissionMode::Verify => panic!("expected admission run mode"),
                }
            }
            other => panic!("expected admission args, got {other:?}"),
        }

        let missing_profile =
            parse(["cargo", "octet", "admit", "--closure"].map(OsString::from)).expect_err("missing profile fails");
        let missing_closure = parse(["cargo", "octet", "admit", "--profile", "deterministic-core"].map(OsString::from))
            .expect_err("missing closure fails");
        assert!(missing_profile.contains("requires `--profile deterministic-core`"));
        assert!(missing_closure.contains("requires explicit `--closure`"));
    }

    // r[verify octet.effect_closure.evidence.tamper]
    #[test]
    fn admission_verify_parse_requires_artifact_dir_and_rejects_sarif() {
        let parsed = parse(
            [
                "cargo",
                "octet",
                "admit",
                "verify",
                "--artifact-dir",
                "artifacts/admission",
                "--output-format=json",
            ]
            .map(OsString::from),
        )
        .expect("parse admission verification");
        match parsed {
            CommandArgs::Admit(args) => {
                assert_eq!(args.output_format, OutputFormat::Json);
                assert!(matches!(args.mode, AdmissionMode::Verify));
            }
            other => panic!("expected admission args, got {other:?}"),
        }

        let missing = parse(["cargo", "octet", "admit", "verify"].map(OsString::from))
            .expect_err("missing artifact directory fails");
        let sarif = parse(
            [
                "cargo",
                "octet",
                "admit",
                "verify",
                "--artifact-dir",
                "artifacts/admission",
                "--output-format=sarif",
            ]
            .map(OsString::from),
        )
        .expect_err("SARIF verification fails");
        assert!(missing.contains("requires `--artifact-dir PATH`"));
        assert!(sarif.contains("supports human or json"));
    }

    #[test]
    fn check_parse_accepts_function_address_adoption_options() {
        let parsed = parse(
            [
                "cargo",
                "octet",
                "check",
                "--function-addresses=warn",
                "--function-address-path",
                "crates/core/src",
                "--function-address-baseline=artifacts/function-addresses.json",
                "--",
                "--all-targets",
            ]
            .map(OsString::from),
        )
        .expect("parse function-address check options");

        match parsed {
            CommandArgs::Check(args) => {
                assert_eq!(args.function_addresses.mode, crate::function_address::FunctionAddressMode::Warn);
                assert_eq!(args.function_addresses.paths, vec![std::path::PathBuf::from("crates/core/src")]);
                assert_eq!(
                    args.function_addresses.baseline,
                    Some(std::path::PathBuf::from("artifacts/function-addresses.json"))
                );
                assert_eq!(args.cargo_check_args.as_deref(), Some(["--all-targets".to_owned()].as_slice()));
            }
            other => panic!("expected check args, got {other:?}"),
        }

        let error = parse(["cargo", "octet", "check", "--function-addresses=strict"].map(OsString::from))
            .expect_err("invalid mode fails");
        assert!(error.contains("unsupported function-address mode"));
    }

    #[test]
    fn artifact_verify_parse_accepts_dir_json_report_and_valence_export() {
        let parsed = parse(
            [
                "cargo",
                "octet",
                "artifact",
                "verify",
                "--artifact-dir",
                "artifacts/octet",
                "--output-format",
                "json",
                "--emit-valence-provenance",
                "artifacts/octet/valence-provenance.json",
            ]
            .map(OsString::from),
        )
        .expect("parse artifact verify args");

        match parsed {
            CommandArgs::Artifact(args) => {
                assert_eq!(args.artifact_dir, Path::new("artifacts/octet"));
                assert_eq!(args.output_format, OutputFormat::Json);
                assert_eq!(
                    args.emit_valence_provenance.as_deref(),
                    Some(Path::new("artifacts/octet/valence-provenance.json"))
                );
            }
            other => panic!("expected artifact args, got {other:?}"),
        }

        let missing_dir = parse(["cargo", "octet", "artifact", "verify"].map(OsString::from))
            .expect_err("artifact verify requires artifact dir");
        assert!(missing_dir.contains("artifact verify requires `--artifact-dir PATH`"));

        let missing_emit_path = parse(
            [
                "cargo",
                "octet",
                "artifact",
                "verify",
                "--artifact-dir",
                "artifacts/octet",
                "--emit-valence-provenance",
            ]
            .map(OsString::from),
        )
        .expect_err("artifact verify emit requires output path");
        assert!(missing_emit_path.contains("missing value after `--emit-valence-provenance`"));
    }

    #[test]
    fn starter_check_parse_stays_separate_from_dynamic_evidence() {
        let parsed = parse(["cargo", "octet", "check"].map(OsString::from)).expect("parse starter check");
        match parsed {
            CommandArgs::Check(args) => {
                assert!(args.scope_args.is_empty());
                assert_eq!(args.output_format, OutputFormat::Human);
                assert_eq!(args.command_line, "cargo octet check");
            }
            other => panic!("expected starter check args, got {other:?}"),
        }
    }

    #[test]
    fn evidence_index_codebase_parse_accepts_build_query_and_html() {
        let build = parse(
            [
                "cargo",
                "octet",
                "evidence-index",
                "codebase",
                "build",
                "--artifact",
                "artifacts/octet/findings.jsonl",
                "--artifact-dir=artifacts/proofs",
                "--output",
                "artifacts/octet/evidence-index.json",
                "--advisory",
            ]
            .map(OsString::from),
        )
        .expect("parse evidence codebase build");
        match build {
            CommandArgs::EvidenceIndex(args) => {
                assert_eq!(args.mode, EvidenceIndexMode::CodebaseBuild);
                assert_eq!(args.artifacts, vec![std::path::PathBuf::from("artifacts/octet/findings.jsonl")]);
                assert_eq!(args.artifact_dirs, vec![std::path::PathBuf::from("artifacts/proofs")]);
                assert_eq!(args.output, Some(std::path::PathBuf::from("artifacts/octet/evidence-index.json")));
                assert!(args.advisory);
            }
            other => panic!("expected evidence index build args, got {other:?}"),
        }

        let query = parse(
            [
                "cargo",
                "octet",
                "evidence-index",
                "codebase",
                "query",
                "--index",
                "artifacts/octet/evidence-index.json",
                "--identity=r[octet.example]",
            ]
            .map(OsString::from),
        )
        .expect("parse evidence codebase query");
        match query {
            CommandArgs::EvidenceIndex(args) => {
                assert_eq!(args.mode, EvidenceIndexMode::CodebaseQuery);
                assert_eq!(args.index, Some(std::path::PathBuf::from("artifacts/octet/evidence-index.json")));
                assert_eq!(args.identity.as_deref(), Some("r[octet.example]"));
            }
            other => panic!("expected evidence index query args, got {other:?}"),
        }

        let html = parse(
            [
                "cargo",
                "octet",
                "evidence-index",
                "codebase",
                "html",
                "--index=artifacts/octet/evidence-index.json",
                "--output=site/index.html",
            ]
            .map(OsString::from),
        )
        .expect("parse evidence codebase html");
        match html {
            CommandArgs::EvidenceIndex(args) => {
                assert_eq!(args.mode, EvidenceIndexMode::CodebaseHtml);
                assert_eq!(args.index, Some(std::path::PathBuf::from("artifacts/octet/evidence-index.json")));
                assert_eq!(args.output, Some(std::path::PathBuf::from("site/index.html")));
            }
            other => panic!("expected evidence index html args, got {other:?}"),
        }
    }

    #[test]
    fn evidence_index_codebase_parse_rejects_missing_inputs() {
        let build_error = parse(["cargo", "octet", "evidence-index", "codebase", "build"].map(OsString::from))
            .expect_err("build requires inputs");
        assert!(build_error.contains("requires at least one `--artifact` or `--artifact-dir`"));

        let query_error = parse(
            [
                "cargo",
                "octet",
                "evidence-index",
                "codebase",
                "query",
                "--index",
                "index.json",
            ]
            .map(OsString::from),
        )
        .expect_err("query requires identity");
        assert!(query_error.contains("requires `--index PATH --identity ID`"));
    }

    #[test]
    fn evidence_collect_parse_accepts_import_bundle_options_and_rejects_sarif() {
        let parsed = parse(
            [
                "cargo",
                "octet",
                "evidence",
                "collect",
                "--rail",
                "dependency-audit",
                "--input",
                "deny.json",
                "--artifact-dir=artifacts/dependency-audit",
                "--workspace-root",
                ".",
                "--config",
                "deny.toml",
                "--locked",
                "--parent-artifact",
                "artifacts/octet/status.json",
                "--output-format",
                "json",
            ]
            .map(OsString::from),
        )
        .expect("parse evidence collect args");

        match parsed {
            CommandArgs::EvidenceRail(args) => {
                assert_eq!(args.rail, crate::evidence_rails::EvidenceRailKind::DependencyAudit);
                assert_eq!(args.input, Some(std::path::PathBuf::from("deny.json")));
                assert_eq!(args.artifact_dir, std::path::PathBuf::from("artifacts/dependency-audit"));
                assert_eq!(args.workspace_root, Some(std::path::PathBuf::from(".")));
                assert_eq!(args.config, Some(std::path::PathBuf::from("deny.toml")));
                assert_eq!(args.parent_artifacts, vec!["artifacts/octet/status.json".to_owned()]);
                assert_eq!(args.output_format, OutputFormat::Json);
                assert!(args.locked);
            }
            other => panic!("expected evidence rail args, got {other:?}"),
        }

        let sarif = parse(
            [
                "cargo",
                "octet",
                "evidence",
                "collect",
                "--rail=clippy",
                "--input=clippy.jsonl",
                "--output-format=sarif",
            ]
            .map(OsString::from),
        )
        .expect_err("evidence collect rejects SARIF");
        assert!(sarif.contains("does not support SARIF"));
    }

    #[test]
    fn spacewasm_collect_parse_accepts_external_reports_and_rejects_missing_path() {
        let parsed = parse(
            [
                "cargo",
                "octet",
                "evidence",
                "collect",
                "--rail=wasm-artifact",
                "--input=module.wasm",
                "--config=standards/wasm-artifact/generated/profiles.json",
                "--profile=spacewasm-mvp",
                "--external-report=chaoscontrol.json",
            ]
            .map(OsString::from),
        )
        .expect("parse SpaceWasm evidence collect args");

        match parsed {
            CommandArgs::EvidenceRail(args) => {
                assert_eq!(args.profile.as_deref(), Some("spacewasm-mvp"));
                assert_eq!(args.external_reports, vec![std::path::PathBuf::from("chaoscontrol.json")]);
            }
            other => panic!("expected evidence rail args, got {other:?}"),
        }

        let missing = parse(
            [
                "cargo",
                "octet",
                "evidence",
                "collect",
                "--rail=wasm-artifact",
                "--external-report",
            ]
            .map(OsString::from),
        )
        .expect_err("missing external report path fails");
        assert!(missing.contains("missing value after `--external-report`"));
    }

    #[test]
    fn migrate_session_parse_accepts_apply_manifest_and_dry_run() {
        let parsed = parse(
            [
                "cargo",
                "octet",
                "migrate",
                "session",
                "apply",
                "--session",
                "migration-session.json",
                "--operation-manifest=operation-manifest.json",
                "--output-format",
                "json",
                "--dry-run",
            ]
            .map(OsString::from),
        )
        .expect("parse migrate session apply");

        match parsed {
            CommandArgs::Migrate(args) => {
                assert_eq!(args.mode, MigrateMode::SessionApply);
                assert_eq!(args.session, Some(std::path::PathBuf::from("migration-session.json")));
                assert_eq!(args.operation_manifest, Some(std::path::PathBuf::from("operation-manifest.json")));
                assert_eq!(args.output_format, OutputFormat::Json);
                assert!(args.dry_run);
            }
            other => panic!("expected migrate session apply args, got {other:?}"),
        }
    }

    #[test]
    fn migrate_session_parse_rejects_missing_session_or_manifest() {
        let missing_session = parse(["cargo", "octet", "migrate", "session", "verify"].map(OsString::from))
            .expect_err("session commands require session path");
        assert!(missing_session.contains("migrate session requires `--session PATH`"));

        let missing_manifest = parse(
            [
                "cargo",
                "octet",
                "migrate",
                "session",
                "apply",
                "--session",
                "migration-session.json",
            ]
            .map(OsString::from),
        )
        .expect_err("apply requires operation manifest");
        assert!(missing_manifest.contains("migrate session apply requires `--operation-manifest PATH`"));
    }

    #[test]
    fn fingerprint_parse_accepts_default_check() {
        let parsed =
            parse(["cargo", "octet", "fingerprint", "src/lib.rs"].map(OsString::from)).expect("parse fingerprint args");
        match parsed {
            CommandArgs::Fingerprint(args) => {
                assert_eq!(args.mode, FingerprintMode::Check);
                assert_eq!(args.paths.len(), 1);
            }
            other => panic!("expected fingerprint args, got {other:?}"),
        }
    }

    #[test]
    fn fingerprint_parse_accepts_compatibility_alias() {
        let parsed = parse(["cargo", "tigerstyle", "fingerprint", "src/lib.rs"].map(OsString::from))
            .expect("parse fingerprint args");
        match parsed {
            CommandArgs::Fingerprint(args) => {
                assert_eq!(args.mode, FingerprintMode::Check);
                assert_eq!(args.paths.len(), 1);
            }
            other => panic!("expected fingerprint args, got {other:?}"),
        }
    }

    #[test]
    fn object_parse_accepts_emit_selection() {
        let parsed = parse(
            [
                "cargo",
                "octet",
                "object",
                "emit",
                "--function",
                "checked",
                "src/lib.rs",
                "--output-format",
                "json",
            ]
            .map(OsString::from),
        )
        .expect("parse object emit args");
        match parsed {
            CommandArgs::Object(args) => {
                assert_eq!(args.paths.len(), 1);
                assert_eq!(args.function.as_deref(), Some("checked"));
                assert_eq!(args.output_format, OutputFormat::Json);
            }
            other => panic!("expected object args, got {other:?}"),
        }
    }

    #[test]
    fn provider_manifest_parse_accepts_export() {
        let parsed = parse(
            [
                "cargo",
                "octet",
                "provider-manifest",
                "export",
                "--driver",
                "driver",
                "--lint-library=liboctet.so",
                "--config",
                "dylint.toml",
                "--standards-policy",
                "octet-policies.json",
                "--required-lint",
                "no_unwrap",
            ]
            .map(OsString::from),
        )
        .expect("parse provider manifest export args");
        match parsed {
            CommandArgs::ProviderManifest(args) => match args.mode {
                super::ProviderManifestMode::Export {
                    driver,
                    lint_library,
                    config,
                    standards_policy,
                    required_lints,
                    ..
                } => {
                    assert_eq!(driver, std::path::PathBuf::from("driver"));
                    assert_eq!(lint_library, std::path::PathBuf::from("liboctet.so"));
                    assert_eq!(config, std::path::PathBuf::from("dylint.toml"));
                    assert_eq!(standards_policy, Some(std::path::PathBuf::from("octet-policies.json")));
                    assert_eq!(required_lints, vec!["no_unwrap".to_owned()]);
                }
                super::ProviderManifestMode::Validate { .. } => {
                    panic!("expected provider manifest export mode")
                }
            },
            other => panic!("expected provider manifest args, got {other:?}"),
        }
    }

    #[test]
    fn provider_manifest_parse_rejects_missing_driver() {
        let error = parse(
            [
                "cargo",
                "octet",
                "provider-manifest",
                "export",
                "--lint-library",
                "liboctet.so",
                "--config",
                "dylint.toml",
            ]
            .map(OsString::from),
        )
        .expect_err("missing driver should fail");
        assert_eq!(error, "missing `--driver`");
    }

    #[test]
    fn prove_parse_accepts_rail_selector() {
        let parsed = parse(
            [
                "cargo",
                "octet",
                "prove",
                "--registry",
                "octet-policies.json",
                "--source",
                "src/verified",
                "--rail",
                "octet.verus_proof_rails.addition-bound#rail-0",
                "--rail=octet.verus_proof_rails.ordering-reflexive-production",
                "--output-format",
                "json",
            ]
            .map(OsString::from),
        )
        .expect("parse prove rail selector args");

        match parsed {
            CommandArgs::Prove(args) => {
                assert_eq!(args.registry, Some(std::path::PathBuf::from("octet-policies.json")));
                assert_eq!(args.source, vec![std::path::PathBuf::from("src/verified")]);
                assert_eq!(args.rail, vec![
                    "octet.verus_proof_rails.addition-bound#rail-0".to_owned(),
                    "octet.verus_proof_rails.ordering-reflexive-production".to_owned(),
                ]);
                assert_eq!(args.output_format, OutputFormat::Json);
            }
            other => panic!("expected prove args, got {other:?}"),
        }
    }

    #[test]
    fn proof_ref_parse_accepts_validate_paths_and_json_output() {
        let parsed = parse(
            [
                "cargo",
                "octet",
                "proof-ref",
                "validate",
                "--proof-cards",
                "artifacts/octet/proof-cards.json",
                "--ref=proof-ref.json",
                "--output-format",
                "json",
            ]
            .map(OsString::from),
        )
        .expect("parse proof-ref validate args");

        match parsed {
            CommandArgs::ProofRef(args) => match args.mode {
                ProofRefMode::Validate(validate_args) => {
                    assert_eq!(validate_args.proof_cards, std::path::PathBuf::from("artifacts/octet/proof-cards.json"));
                    assert_eq!(validate_args.reference, std::path::PathBuf::from("proof-ref.json"));
                    assert_eq!(validate_args.output_format, OutputFormat::Json);
                }
                ProofRefMode::Scaffold(_) => panic!("expected proof-ref validate mode"),
            },
            other => panic!("expected proof-ref args, got {other:?}"),
        }
    }

    #[test]
    fn proof_ref_parse_accepts_scaffold_args() {
        let parsed = parse(
            [
                "cargo",
                "octet",
                "proof-ref",
                "scaffold",
                "--proof-cards",
                "artifacts/octet/proof-cards.json",
                "--proof-or-property-id=octet.verus#rail-0",
                "--reference-id",
                "unchecked-narrowing-range-proof",
                "--consumer-context",
                "allow tigerstyle::unchecked_narrowing at src/codec.rs:42",
                "--consumer-source=src/codec.rs",
                "--anchor",
                "octet-proof-ref: unchecked-narrowing-range-proof -> octet.verus#rail-0",
                "--output",
                "proof-ref.json",
            ]
            .map(OsString::from),
        )
        .expect("parse proof-ref scaffold args");

        match parsed {
            CommandArgs::ProofRef(args) => match args.mode {
                ProofRefMode::Scaffold(scaffold_args) => {
                    assert_eq!(scaffold_args.proof_cards, std::path::PathBuf::from("artifacts/octet/proof-cards.json"));
                    assert_eq!(scaffold_args.proof_or_property_id, "octet.verus#rail-0");
                    assert_eq!(scaffold_args.reference_id, "unchecked-narrowing-range-proof");
                    assert_eq!(scaffold_args.consumer_source, std::path::PathBuf::from("src/codec.rs"));
                    assert_eq!(scaffold_args.output, Some(std::path::PathBuf::from("proof-ref.json")));
                }
                ProofRefMode::Validate(_) => panic!("expected proof-ref scaffold mode"),
            },
            other => panic!("expected proof-ref args, got {other:?}"),
        }
    }

    #[test]
    fn proof_ref_parse_rejects_sarif_and_help_is_bounded() {
        let error = parse(
            [
                "cargo",
                "octet",
                "proof-ref",
                "validate",
                "--proof-cards",
                "proof-cards.json",
                "--ref",
                "proof-ref.json",
                "--output-format",
                "sarif",
            ]
            .map(OsString::from),
        )
        .expect_err("proof-ref rejects sarif");
        assert!(error.contains("proof-ref does not support SARIF"));

        let help = parse(["cargo", "octet", "proof-ref", "--help"].map(OsString::from)).expect("parse proof-ref help");
        assert!(matches!(help, CommandArgs::Help(HelpTopic::ProofRef)));
        assert!(
            super::help_text(HelpTopic::ProofRef)
                .contains("does not introduce a consumer-implementable Verified trait")
        );
    }

    #[test]
    fn object_parse_accepts_corpus_receipt() {
        let parsed = parse(
            [
                "cargo",
                "octet",
                "object",
                "corpus",
                "receipt",
                "--output",
                "receipt.json",
                "cargo-octet/src/object.rs",
            ]
            .map(OsString::from),
        )
        .expect("parse object corpus receipt args");
        match parsed {
            CommandArgs::Object(args) => {
                assert_eq!(args.paths.len(), 1);
                assert_eq!(args.output_format, OutputFormat::Json);
                match args.mode {
                    super::ObjectMode::CorpusReceipt { output } => {
                        assert_eq!(output, std::path::PathBuf::from("receipt.json"));
                    }
                    other => panic!("expected corpus receipt mode, got {other:?}"),
                }
            }
            other => panic!("expected object args, got {other:?}"),
        }
    }

    #[test]
    fn object_parse_accepts_corpus_contract() {
        let parsed = parse(
            [
                "cargo",
                "octet",
                "object",
                "corpus",
                "contract",
                "--output-format",
                "json",
            ]
            .map(OsString::from),
        )
        .expect("parse object corpus contract args");
        match parsed {
            CommandArgs::Object(args) => {
                assert!(matches!(args.mode, super::ObjectMode::CorpusContract));
                assert_eq!(args.output_format, OutputFormat::Json);
                assert!(args.paths.is_empty());
            }
            other => panic!("expected object args, got {other:?}"),
        }
    }

    #[test]
    fn object_parse_accepts_corpus_caveats() {
        let parsed = parse(
            [
                "cargo",
                "octet",
                "object",
                "corpus",
                "caveats",
                "--output-format",
                "json",
                "cargo-octet/src/object.rs",
            ]
            .map(OsString::from),
        )
        .expect("parse object corpus caveats args");
        match parsed {
            CommandArgs::Object(args) => {
                assert!(matches!(args.mode, super::ObjectMode::CorpusCaveats));
                assert_eq!(args.output_format, OutputFormat::Json);
                assert_eq!(args.paths.len(), 1);
            }
            other => panic!("expected object args, got {other:?}"),
        }
    }

    #[test]
    fn object_parse_accepts_corpus_check() {
        let parsed = parse(
            [
                "cargo",
                "octet",
                "object",
                "corpus",
                "check",
                "--receipt",
                "receipt.json",
                "--output-format",
                "json",
                "cargo-octet/src/object.rs",
            ]
            .map(OsString::from),
        )
        .expect("parse object corpus check args");
        match parsed {
            CommandArgs::Object(args) => {
                assert_eq!(args.paths.len(), 1);
                assert_eq!(args.output_format, OutputFormat::Json);
                match args.mode {
                    super::ObjectMode::CorpusCheck { receipt } => {
                        assert_eq!(receipt, std::path::PathBuf::from("receipt.json"));
                    }
                    other => panic!("expected corpus check mode, got {other:?}"),
                }
            }
            other => panic!("expected object args, got {other:?}"),
        }
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn object_parse_accepts_supplement_emit_and_check() {
        let emit = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "emit",
                "--family",
                "hir",
                "--function",
                "checked",
                "src/lib.rs",
                "--output-format",
                "json",
            ]
            .map(OsString::from),
        )
        .expect("parse supplement emit args");
        match emit {
            CommandArgs::Object(args) => {
                assert_eq!(args.function.as_deref(), Some("checked"));
                assert_eq!(args.paths.len(), 1);
                assert_eq!(args.output_format, OutputFormat::Json);
                assert!(matches!(args.mode, super::ObjectMode::SupplementEmit {
                    family: super::SupplementFamilyArg::Hir,
                    scope: super::SupplementScopeArg::Pilot,
                }));
            }
            other => panic!("expected object supplement emit args, got {other:?}"),
        }

        let check = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "check",
                "--family=mir",
                "--receipt",
                "supplement.json",
                "src/lib.rs",
            ]
            .map(OsString::from),
        )
        .expect("parse supplement check args");
        match check {
            CommandArgs::Object(args) => match args.mode {
                super::ObjectMode::SupplementCheck { family, scope, receipt } => {
                    assert_eq!(family, super::SupplementFamilyArg::Mir);
                    assert_eq!(scope, super::SupplementScopeArg::Pilot);
                    assert_eq!(receipt, std::path::PathBuf::from("supplement.json"));
                }
                other => panic!("expected supplement check mode, got {other:?}"),
            },
            other => panic!("expected object supplement check args, got {other:?}"),
        }

        let thir = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "emit",
                "--family=thir",
                "--function",
                "checked",
                "src/lib.rs",
            ]
            .map(OsString::from),
        )
        .expect("parse THIR supplement emit args");
        match thir {
            CommandArgs::Object(args) => match args.mode {
                super::ObjectMode::SupplementEmit { family, scope } => {
                    assert_eq!(family, super::SupplementFamilyArg::Thir);
                    assert_eq!(scope, super::SupplementScopeArg::Pilot);
                }
                other => panic!("expected THIR supplement emit mode, got {other:?}"),
            },
            other => panic!("expected object THIR supplement args, got {other:?}"),
        }

        let spike = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "emit",
                "--family=hir",
                "--spike=rustc-hir-fixture",
                "--output-format=json",
            ]
            .map(OsString::from),
        )
        .expect("parse rustc HIR spike supplement args");
        match spike {
            CommandArgs::Object(args) => match args.mode {
                super::ObjectMode::SupplementEmit { family, scope } => {
                    assert_eq!(family, super::SupplementFamilyArg::Hir);
                    assert_eq!(scope, super::SupplementScopeArg::RustcHirFixtureSpike);
                }
                other => panic!("expected supplement spike emit mode, got {other:?}"),
            },
            other => panic!("expected object supplement spike args, got {other:?}"),
        }

        let branch_shape = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "emit",
                "--family=hir",
                "--spike=rustc-hir-fixture",
                "--fixture-shape=branch-control-flow",
                "--output-format=json",
            ]
            .map(OsString::from),
        )
        .expect("parse branch fixture shape supplement args");
        match branch_shape {
            CommandArgs::Object(args) => {
                assert_eq!(args.function.as_deref(), Some("clamp_to_limit"));
                assert_eq!(args.paths, vec![std::path::PathBuf::from(
                    "cargo-octet/tests/fixtures/hir-spike/clamp.rs"
                )]);
                assert!(matches!(args.mode, super::ObjectMode::SupplementEmit {
                    family: super::SupplementFamilyArg::Hir,
                    scope: super::SupplementScopeArg::RustcHirFixtureSpike,
                }));
            }
            other => panic!("expected branch fixture shape args, got {other:?}"),
        }

        let shape_without_spike = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "emit",
                "--family=hir",
                "--fixture-shape=branch-control-flow",
            ]
            .map(OsString::from),
        )
        .expect_err("fixture shape requires rustc spike scope");
        assert!(shape_without_spike.contains(
            "requires `--spike rustc-hir-fixture`, `--spike rustc-thir-fixture`, or `--spike rustc-mir-fixture`"
        ));

        let thir_without_shape = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "emit",
                "--family=thir",
                "--spike=rustc-thir-fixture",
                "--output-format=json",
            ]
            .map(OsString::from),
        )
        .expect_err("THIR fixture spike requires explicit fixture shape");
        assert!(
            thir_without_shape
                .contains("requires explicit `--fixture-shape checked-arithmetic`, `--fixture-shape call-return`, or `--fixture-shape branch-control-flow`")
        );

        let thir_unknown_shape = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "emit",
                "--family=thir",
                "--spike=rustc-thir-fixture",
                "--fixture-shape=not-selected",
                "--output-format=json",
            ]
            .map(OsString::from),
        )
        .expect_err("THIR fixture spike rejects non-selected fixture shape");
        assert!(
            thir_unknown_shape.contains(
                "unsupported THIR supplement fixture shape `not-selected`; expected `checked-arithmetic`, `call-return`, or `branch-control-flow`"
            ),
            "unexpected THIR unknown fixture shape error: {thir_unknown_shape}"
        );

        let thir_shape = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "emit",
                "--family=thir",
                "--spike=rustc-thir-fixture",
                "--fixture-shape=checked-arithmetic",
                "--output-format=json",
            ]
            .map(OsString::from),
        )
        .expect("parse THIR fixture shape supplement args");
        match thir_shape {
            CommandArgs::Object(args) => {
                assert_eq!(args.function.as_deref(), Some("checked"));
                assert_eq!(args.paths, vec![std::path::PathBuf::from(
                    "cargo-octet/tests/fixtures/thir-spike/checked.rs"
                )]);
                assert!(matches!(args.mode, super::ObjectMode::SupplementEmit {
                    family: super::SupplementFamilyArg::Thir,
                    scope: super::SupplementScopeArg::RustcThirFixtureSpike,
                }));
            }
            other => panic!("expected THIR fixture shape args, got {other:?}"),
        }

        let thir_call_shape = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "emit",
                "--family=thir",
                "--spike=rustc-thir-fixture",
                "--fixture-shape=call-return",
                "--output-format=json",
            ]
            .map(OsString::from),
        )
        .expect("parse THIR call-return fixture shape supplement args");
        match thir_call_shape {
            CommandArgs::Object(args) => {
                assert_eq!(args.function.as_deref(), Some("call_then_return"));
                assert_eq!(args.paths, vec![std::path::PathBuf::from(
                    "cargo-octet/tests/fixtures/thir-spike/call_return.rs"
                )]);
                assert!(matches!(args.mode, super::ObjectMode::SupplementEmit {
                    family: super::SupplementFamilyArg::Thir,
                    scope: super::SupplementScopeArg::RustcThirFixtureSpike,
                }));
            }
            other => panic!("expected THIR call-return fixture shape args, got {other:?}"),
        }

        let thir_branch_shape = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "emit",
                "--family=thir",
                "--spike=rustc-thir-fixture",
                "--fixture-shape=branch-control-flow",
                "--output-format=json",
            ]
            .map(OsString::from),
        )
        .expect("parse THIR branch-control-flow fixture shape supplement args");
        match thir_branch_shape {
            CommandArgs::Object(args) => {
                assert_eq!(args.function.as_deref(), Some("choose_branch"));
                assert_eq!(args.paths, vec![std::path::PathBuf::from(
                    "cargo-octet/tests/fixtures/thir-spike/branch_control_flow.rs"
                )]);
                assert!(matches!(args.mode, super::ObjectMode::SupplementEmit {
                    family: super::SupplementFamilyArg::Thir,
                    scope: super::SupplementScopeArg::RustcThirFixtureSpike,
                }));
            }
            other => panic!("expected THIR branch-control-flow fixture shape args, got {other:?}"),
        }
        let mir_shape = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "emit",
                "--family=mir",
                "--spike=rustc-mir-fixture",
                "--fixture-shape=checked-arithmetic",
                "--output-format=json",
            ]
            .map(OsString::from),
        )
        .expect("parse MIR fixture shape supplement args");
        match mir_shape {
            CommandArgs::Object(args) => {
                assert_eq!(args.function.as_deref(), Some("checked"));
                assert_eq!(args.paths, vec![std::path::PathBuf::from(
                    "cargo-octet/tests/fixtures/mir-spike/checked.rs"
                )]);
                assert!(matches!(args.mode, super::ObjectMode::SupplementEmit {
                    family: super::SupplementFamilyArg::Mir,
                    scope: super::SupplementScopeArg::RustcMirFixtureSpike,
                }));
            }
            other => panic!("expected MIR fixture shape args, got {other:?}"),
        }

        let mir_without_shape = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "emit",
                "--family=mir",
                "--spike=rustc-mir-fixture",
                "--output-format=json",
            ]
            .map(OsString::from),
        )
        .expect_err("MIR fixture spike requires explicit fixture shape");
        assert!(
            mir_without_shape
                .contains("requires explicit `--fixture-shape checked-arithmetic`, `--fixture-shape call-return`, or `--fixture-shape branch-control-flow`")
        );

        let mir_checked_alias = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "emit",
                "--family=mir",
                "--spike=rustc-mir-fixture",
                "--fixture-shape=checked",
                "--output-format=json",
            ]
            .map(OsString::from),
        )
        .expect_err("MIR fixture spike rejects aliases outside inventory selector");
        assert!(
            mir_checked_alias
                .contains("supports only `--fixture-shape checked-arithmetic`, `--fixture-shape call-return`, or `--fixture-shape branch-control-flow`"),
            "unexpected MIR checked alias fixture shape error: {mir_checked_alias}"
        );

        let mir_call_shape = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "emit",
                "--family=mir",
                "--spike=rustc-mir-fixture",
                "--fixture-shape=call-return",
                "--output-format=json",
            ]
            .map(OsString::from),
        )
        .expect("parse MIR call-return fixture shape supplement args");
        match mir_call_shape {
            CommandArgs::Object(args) => {
                assert_eq!(args.function.as_deref(), Some("call_then_return"));
                assert_eq!(args.paths, vec![std::path::PathBuf::from(
                    "cargo-octet/tests/fixtures/mir-spike/call_return.rs"
                )]);
                assert!(matches!(args.mode, super::ObjectMode::SupplementEmit {
                    family: super::SupplementFamilyArg::Mir,
                    scope: super::SupplementScopeArg::RustcMirFixtureSpike,
                }));
            }
            other => panic!("expected MIR call-return fixture shape args, got {other:?}"),
        }

        let mir_branch_shape = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "emit",
                "--family=mir",
                "--spike=rustc-mir-fixture",
                "--fixture-shape=branch-control-flow",
                "--output-format=json",
            ]
            .map(OsString::from),
        )
        .expect("parse MIR branch-control-flow fixture shape supplement args");
        match mir_branch_shape {
            CommandArgs::Object(args) => {
                assert_eq!(args.function.as_deref(), Some("choose_branch"));
                assert_eq!(args.paths, vec![std::path::PathBuf::from(
                    "cargo-octet/tests/fixtures/mir-spike/branch_control_flow.rs"
                )]);
                assert!(matches!(args.mode, super::ObjectMode::SupplementEmit {
                    family: super::SupplementFamilyArg::Mir,
                    scope: super::SupplementScopeArg::RustcMirFixtureSpike,
                }));
            }
            other => panic!("expected MIR branch-control-flow fixture shape args, got {other:?}"),
        }
    }

    #[test]
    fn object_parse_accepts_supplement_fixture_inventory() {
        let inventory = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "fixtures",
                "--family=hir",
                "--spike=rustc-hir-fixture",
                "--output-format=json",
            ]
            .map(OsString::from),
        )
        .expect("parse supplement fixture inventory args");
        match inventory {
            CommandArgs::Object(args) => {
                assert_eq!(args.paths, Vec::<std::path::PathBuf>::new());
                assert_eq!(args.function, None);
                assert_eq!(args.output_format, OutputFormat::Json);
                assert!(matches!(args.mode, super::ObjectMode::SupplementFixtures {
                    family: super::SupplementFamilyArg::Hir,
                    scope: super::SupplementScopeArg::RustcHirFixtureSpike,
                }));
            }
            other => panic!("expected object supplement fixtures args, got {other:?}"),
        }

        let missing_scope =
            parse(["cargo", "octet", "object", "supplement", "fixtures", "--family=hir"].map(OsString::from))
                .expect_err("fixture inventory requires rustc spike scope");
        assert!(missing_scope.contains("requires `--spike rustc-hir-fixture`"));

        let thir_inventory = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "fixtures",
                "--family=thir",
                "--spike=rustc-thir-fixture",
                "--output-format=json",
            ]
            .map(OsString::from),
        )
        .expect("parse THIR supplement fixture inventory args");
        match thir_inventory {
            CommandArgs::Object(args) => {
                assert_eq!(args.output_format, OutputFormat::Json);
                assert!(matches!(args.mode, super::ObjectMode::SupplementFixtures {
                    family: super::SupplementFamilyArg::Thir,
                    scope: super::SupplementScopeArg::RustcThirFixtureSpike,
                }));
            }
            other => panic!("expected THIR object supplement fixtures args, got {other:?}"),
        }

        let mir_inventory = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "fixtures",
                "--family=mir",
                "--spike=rustc-mir-fixture",
                "--output-format=json",
            ]
            .map(OsString::from),
        )
        .expect("parse MIR supplement fixture inventory args");
        match mir_inventory {
            CommandArgs::Object(args) => {
                assert_eq!(args.output_format, OutputFormat::Json);
                assert!(matches!(args.mode, super::ObjectMode::SupplementFixtures {
                    family: super::SupplementFamilyArg::Mir,
                    scope: super::SupplementScopeArg::RustcMirFixtureSpike,
                }));
            }
            other => panic!("expected MIR object supplement fixtures args, got {other:?}"),
        }

        let mir_wrong_scope = parse(
            [
                "cargo",
                "octet",
                "object",
                "supplement",
                "fixtures",
                "--family=mir",
                "--spike=rustc-hir-fixture",
            ]
            .map(OsString::from),
        )
        .expect_err("MIR fixture inventory rejects HIR scope");
        assert!(mir_wrong_scope.contains("requires `--spike rustc-mir-fixture`"));
    }

    #[test]
    fn help_parse_accepts_root_and_subcommand_help() {
        let root = parse(["cargo", "octet", "--help"].map(OsString::from)).expect("parse root help");
        assert!(matches!(root, CommandArgs::Help(HelpTopic::Root)));

        let artifact =
            parse(["cargo", "octet", "artifact", "verify", "--help"].map(OsString::from)).expect("parse artifact help");
        assert!(matches!(artifact, CommandArgs::Help(HelpTopic::Artifact)));

        let check = parse(["cargo", "octet", "check", "--help"].map(OsString::from)).expect("parse check help");
        assert!(matches!(check, CommandArgs::Help(HelpTopic::Check)));

        for args in [
            &["cargo", "octet", "object", "--help"][..],
            &["cargo", "octet", "object", "supplement", "fixtures", "--help"][..],
            &["cargo", "octet", "object", "supplement", "emit", "--help"][..],
            &["cargo", "octet", "object", "supplement", "check", "--help"][..],
        ] {
            let parsed = parse(args.iter().map(OsString::from)).expect("parse object help");
            assert!(matches!(parsed, CommandArgs::Help(HelpTopic::Object)));
        }
    }

    #[test]
    fn version_parse_accepts_root_version_aliases() {
        for args in [
            ["cargo", "octet", "--version"],
            ["cargo", "octet", "-V"],
            ["cargo", "octet", "version"],
            ["cargo", "tigerstyle", "--version"],
        ] {
            let parsed = parse(args.map(OsString::from)).expect("parse version");
            assert!(matches!(parsed, CommandArgs::Version));
        }
    }

    #[test]
    fn command_line_quoting_matches_contract() {
        assert_eq!(format_command_line("cargo", &["octet".to_owned(), "check".to_owned()]), "cargo octet check");
        assert_eq!(format_command_line("cargo", &["arg with spaces".to_owned()]), "cargo 'arg with spaces'");
        assert_eq!(format_command_line("cargo", &["can't".to_owned()]), "cargo 'can'\"'\"'t'");
        assert_eq!(format_command_line("cargo", &[String::new()]), "cargo ''");
    }

    #[test]
    fn compatibility_alias_command_line_records_octet_canonical_identity() {
        let parsed = parse(["cargo", "slotcar", "check", "--output-format", "json"].map(OsString::from))
            .expect("parse alias check args");
        match parsed {
            CommandArgs::Check(args) => assert_eq!(args.command_line, "cargo octet check --output-format json"),
            other => panic!("expected check args, got {other:?}"),
        }
    }
}
