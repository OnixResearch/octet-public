use super::*;

const MAX_DIAGNOSTIC_COUNT: usize = 32;

struct Diagnostics {
    entries: Vec<String>,
}

impl Diagnostics {
    fn new() -> Self {
        Self {
            entries: Vec::with_capacity(MAX_DIAGNOSTIC_COUNT),
        }
    }

    fn push(&mut self, diagnostic: impl Into<String>) {
        if self.entries.len() < MAX_DIAGNOSTIC_COUNT {
            self.entries.push(diagnostic.into());
        }
    }

    fn into_entries(self) -> Vec<String> {
        self.entries
    }
}

// r[impl octet.dynamic_evidence.cargo_fuzz_receipt]
// r[impl octet.dynamic_evidence.cargo_fuzz_boundary]
pub fn validate(details: &ReceiptDetails) -> Vec<String> {
    let mut diagnostics = Diagnostics::new();
    check_exact_fields(details, &mut diagnostics);
    check_identities(details, &mut diagnostics);
    check_budget(details, &mut diagnostics);
    check_artifacts(details, &mut diagnostics);
    check_outcome(details, &mut diagnostics);
    check_claims(details, &mut diagnostics);
    diagnostics.into_entries()
}

pub fn corpus_root(rows: &[ArtifactIdentity]) -> String {
    let mut canonical = rows.to_vec();
    canonical.sort();
    let mut hasher = blake3::Hasher::new();
    hasher.update(CORPUS_DOMAIN);
    for row in canonical {
        hash_framed(&mut hasher, row.path.as_bytes());
        hash_framed(&mut hasher, &row.length.to_le_bytes());
        hash_framed(&mut hasher, row.blake3.as_bytes());
    }
    format!("b3:{}", hasher.finalize().to_hex())
}

fn check_exact_fields(details: &ReceiptDetails, diagnostics: &mut Diagnostics) {
    push_if(
        diagnostics,
        details.schema_version != RECEIPT_SCHEMA_VERSION,
        "cargo-fuzz receipt schema is unsupported",
    );
    push_if(diagnostics, details.profile_id != PROFILE_ID, "cargo-fuzz receipt profile identity is unsupported");
    push_if(
        diagnostics,
        details.target != TARGET,
        "cargo-fuzz receipt target does not match the reviewed profile",
    );
    push_if(diagnostics, details.engine != ENGINE, "cargo-fuzz receipt engine must be libfuzzer");
    push_if(
        diagnostics,
        details.toolchain != TOOLCHAIN,
        "cargo-fuzz receipt toolchain does not match the reviewed profile",
    );
    push_if(
        diagnostics,
        details.sanitizer != SANITIZER,
        "cargo-fuzz receipt sanitizer does not match the reviewed profile",
    );
    push_if(
        diagnostics,
        details.command_argv != command_argv(),
        "cargo-fuzz receipt command argv does not match the reviewed profile",
    );
    push_if(
        diagnostics,
        details.evidence_tier != EVIDENCE_TIER,
        "cargo-fuzz evidence tier must remain transcript_assertion",
    );
}

fn check_identities(details: &ReceiptDetails, diagnostics: &mut Diagnostics) {
    push_if(
        diagnostics,
        details.profile_identity != EXPECTED_PROFILE_IDENTITY,
        "cargo-fuzz profile identity does not match the generated Nickel registry",
    );
    for (label, identity) in [
        ("profile", details.profile_identity.as_str()),
        ("compiler", details.compiler_identity.as_str()),
        ("bounded log", details.bounded_log_identity.as_str()),
    ] {
        push_if(diagnostics, !is_b3_identity(identity), format!("cargo-fuzz {label} identity must be BLAKE3"));
    }
}

fn check_budget(details: &ReceiptDetails, diagnostics: &mut Diagnostics) {
    let budget = &details.budget;
    push_if(
        diagnostics,
        budget.max_total_time_seconds == 0 || budget.max_total_time_seconds > MAX_TOTAL_TIME_SECONDS,
        "cargo-fuzz time budget is outside the reviewed bound",
    );
    push_if(
        diagnostics,
        budget.max_runs == 0 || budget.max_runs > MAX_RUNS,
        "cargo-fuzz run budget is outside the reviewed bound",
    );
    push_if(diagnostics, budget.seed != FIXED_SEED, "cargo-fuzz seed does not match the reviewed profile");
}

fn check_artifacts(details: &ReceiptDetails, diagnostics: &mut Diagnostics) {
    validate_rows("initial corpus", &details.initial_corpus, diagnostics);
    validate_rows("final corpus", &details.final_corpus, diagnostics);
    push_if(
        diagnostics,
        details.initial_corpus_root != corpus_root(&details.initial_corpus),
        "cargo-fuzz initial corpus root is stale",
    );
    push_if(
        diagnostics,
        details.final_corpus_root != corpus_root(&details.final_corpus),
        "cargo-fuzz final corpus root is stale",
    );
    for (label, artifact) in [
        ("dictionary", details.dictionary.as_ref()),
        ("crash input", details.crash_input.as_ref()),
        ("minimized reproducer", details.minimized_reproducer.as_ref()),
    ] {
        if let Some(artifact) = artifact {
            validate_rows(label, std::slice::from_ref(artifact), diagnostics);
        }
    }
}

fn validate_rows(label: &str, rows: &[ArtifactIdentity], diagnostics: &mut Diagnostics) {
    push_if(
        diagnostics,
        rows.len() > MAX_ARTIFACT_COUNT,
        format!("cargo-fuzz {label} exceeds artifact count limit"),
    );
    let total = rows.iter().try_fold(0_u64, |sum, row| sum.checked_add(row.length));
    push_if(
        diagnostics,
        total.is_none_or(|value| value > MAX_ARTIFACT_BYTES),
        format!("cargo-fuzz {label} exceeds artifact byte limit"),
    );
    let mut paths = std::collections::BTreeSet::new();
    for row in rows {
        push_if(
            diagnostics,
            !safe_relative_path(&row.path),
            format!("cargo-fuzz {label} path is unsafe: {}", row.path),
        );
        push_if(
            diagnostics,
            !paths.insert(row.path.as_str()),
            format!("cargo-fuzz {label} contains duplicate path: {}", row.path),
        );
        push_if(
            diagnostics,
            !is_b3_identity(&row.blake3),
            format!("cargo-fuzz {label} artifact identity must be BLAKE3"),
        );
    }
}

fn check_outcome(details: &ReceiptDetails, diagnostics: &mut Diagnostics) {
    match details.outcome.as_str() {
        "clean" => {
            push_if(
                diagnostics,
                details.crash_input.is_some() || details.minimized_reproducer.is_some(),
                "clean cargo-fuzz receipt must not contain crash artifacts",
            );
            push_if(
                diagnostics,
                !details.replay_argv.is_empty(),
                "clean cargo-fuzz receipt must not contain replay argv",
            );
        }
        "crash" => check_crash_outcome(details, diagnostics),
        "tool_failure" => diagnostics.push("cargo-fuzz tool execution failed"),
        _ => diagnostics.push("cargo-fuzz receipt outcome is unsupported"),
    }
}

fn check_crash_outcome(details: &ReceiptDetails, diagnostics: &mut Diagnostics) {
    push_if(
        diagnostics,
        details.crash_input.is_none(),
        "crash cargo-fuzz receipt is missing the original crash input",
    );
    push_if(
        diagnostics,
        details.minimized_reproducer.is_none(),
        "crash cargo-fuzz receipt is missing the minimized reproducer",
    );
    if let Some(reproducer) = details.minimized_reproducer.as_ref() {
        let expected = vec![
            "cargo".to_owned(),
            "fuzz".to_owned(),
            "run".to_owned(),
            TARGET.to_owned(),
            reproducer.path.clone(),
        ];
        push_if(
            diagnostics,
            details.replay_argv != expected,
            "cargo-fuzz replay argv does not match the minimized reproducer",
        );
    }
}

fn check_claims(details: &ReceiptDetails, diagnostics: &mut Diagnostics) {
    push_if(
        diagnostics,
        !details.claimed_properties.is_empty(),
        "cargo-fuzz receipt must not promote bounded observations into proof claims",
    );
    push_if(
        diagnostics,
        !details.non_claims.iter().any(|value| value == REQUIRED_NON_CLAIM),
        "cargo-fuzz receipt is missing the required non-claim boundary",
    );
}

fn safe_relative_path(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= MAX_PATH_BYTES
        && !path.starts_with('/')
        && !path.starts_with('\\')
        && path.split(['/', '\\']).all(|component| !matches!(component, "" | "." | ".."))
}

fn is_b3_identity(value: &str) -> bool {
    value.strip_prefix("b3:").is_some_and(|digest| {
        digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    })
}

fn hash_framed(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    hasher.update(&bytes.len().to_le_bytes());
    hasher.update(bytes);
}

fn push_if(diagnostics: &mut Diagnostics, condition: bool, diagnostic: impl Into<String>) {
    if condition {
        diagnostics.push(diagnostic);
    }
}
