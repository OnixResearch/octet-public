use serde::Deserialize;
use serde::Serialize;

const FIELD_COMMAND_ARGV: &str = "command_argv";
const FIELD_TIMEOUT_MS: &str = "timeout_ms";
const FIELD_ENV_ALLOWLIST: &str = "environment_allowlist";
const SHELL_METACHARACTERS: &[char] = &['|', '&', ';', '<', '>', '$', '`'];
#[cfg(test)]
const PROCESS_EFFECT_FAMILY: &str = "process";
#[cfg(test)]
const STATUS_PASSED: &str = "passed";
#[cfg(test)]
const STATUS_FAILED: &str = "failed";
#[cfg(test)]
const STATUS_TIMED_OUT: &str = "timed_out";
#[cfg(test)]
const STATUS_MISSING: &str = "missing";
const FIRST_ENV_CHAR_INDEX: usize = 0;
#[cfg(test)]
const DEFAULT_REMEDIATION: &str = "rerun the shell-owned command with explicit argv, timeout, and captured output";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectFamily {
    Process,
}

impl EffectFamily {
    #[cfg(test)]
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Process => PROCESS_EFFECT_FAMILY,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProcessCapabilityToken {
    pub family: EffectFamily,
}

impl ProcessCapabilityToken {
    #[must_use]
    pub fn process() -> Self {
        Self {
            family: EffectFamily::Process,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProcessCommandPlanInput {
    pub command_argv: Vec<String>,
    pub timeout_ms: u64,
    pub environment_allowlist: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProcessCommandPlan {
    pub capability: ProcessCapabilityToken,
    pub executable: String,
    pub args: Vec<String>,
    pub argv: Vec<String>,
    pub timeout_ms: u64,
    pub environment_allowlist: Vec<String>,
}

#[cfg(test)]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CapturedProcessResult {
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    pub stdout: String,
    pub stderr: String,
}

#[cfg(test)]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProcessEffectOutcome {
    pub status: String,
    pub diagnostics: Vec<String>,
    pub remediation: String,
}

pub fn build_process_command_plan(input: ProcessCommandPlanInput) -> Result<ProcessCommandPlan, Vec<String>> {
    let mut diagnostics = Vec::new();
    if input.command_argv.is_empty() {
        diagnostics.push(format!("effect plan field `{FIELD_COMMAND_ARGV}` must not be empty"));
    }
    if input.timeout_ms == 0 {
        diagnostics.push(format!("effect plan field `{FIELD_TIMEOUT_MS}` must be positive"));
    }
    for arg in &input.command_argv {
        if arg.trim().is_empty() {
            diagnostics.push(format!("effect plan field `{FIELD_COMMAND_ARGV}` contains an empty argv segment"));
        }
    }
    if input.command_argv.first().is_some_and(|executable| executable.chars().any(char::is_whitespace)) {
        diagnostics.push("effect plan refuses whitespace in executable argv segment".to_owned());
    }
    if input
        .command_argv
        .first()
        .is_some_and(|executable| executable.chars().any(|character| SHELL_METACHARACTERS.contains(&character)))
    {
        diagnostics.push("effect plan refuses shell metacharacters in executable argv segment".to_owned());
    }
    diagnostics.extend(environment_allowlist_diagnostics(&input.environment_allowlist));
    diagnostics.sort();
    diagnostics.dedup();
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    let executable = input.command_argv.first().expect("diagnostics reject empty argv").clone();
    let args = input.command_argv.iter().skip(1).cloned().collect::<Vec<_>>();
    let mut environment_allowlist = input.environment_allowlist;
    environment_allowlist.sort();
    environment_allowlist.dedup();
    Ok(ProcessCommandPlan {
        capability: ProcessCapabilityToken::process(),
        executable,
        args,
        argv: input.command_argv,
        timeout_ms: input.timeout_ms,
        environment_allowlist,
    })
}

#[cfg(test)]
#[must_use]
pub fn classify_captured_process_result(
    plan: &ProcessCommandPlan,
    result: Option<&CapturedProcessResult>,
) -> ProcessEffectOutcome {
    let mut diagnostics = Vec::new();
    let status = match result {
        Some(result) if result.timed_out => {
            diagnostics.push(format!("process effect `{}` timed out after {}ms", plan.argv.join(" "), plan.timeout_ms));
            STATUS_TIMED_OUT
        }
        Some(result) if result.exit_code == Some(0) => STATUS_PASSED,
        Some(result) => {
            diagnostics.push(format!(
                "process effect `{}` exited with status {:?}",
                plan.argv.join(" "),
                result.exit_code
            ));
            STATUS_FAILED
        }
        None => {
            diagnostics.push(format!("process effect `{}` has no captured result", plan.argv.join(" ")));
            STATUS_MISSING
        }
    };
    diagnostics.sort();
    diagnostics.dedup();
    ProcessEffectOutcome {
        status: status.to_owned(),
        diagnostics,
        remediation: DEFAULT_REMEDIATION.to_owned(),
    }
}

fn environment_allowlist_diagnostics(names: &[String]) -> Vec<String> {
    let mut diagnostics = Vec::new();
    for name in names {
        if name.trim().is_empty() {
            diagnostics.push(format!("effect plan field `{FIELD_ENV_ALLOWLIST}` contains an empty name"));
            continue;
        }
        if name.trim() != name {
            diagnostics.push(format!(
                "effect plan field `{FIELD_ENV_ALLOWLIST}` entry `{name}` must not have surrounding whitespace"
            ));
        }
        if !is_canonical_env_name(name) {
            diagnostics.push(format!(
                "effect plan field `{FIELD_ENV_ALLOWLIST}` entry `{name}` must be a narrow environment variable name"
            ));
        }
    }
    diagnostics
}

fn is_canonical_env_name(name: &str) -> bool {
    name.chars().enumerate().all(|(index, character)| {
        if index == FIRST_ENV_CHAR_INDEX {
            character.is_ascii_uppercase() || character == '_'
        } else {
            character.is_ascii_uppercase() || character.is_ascii_digit() || character == '_'
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXE: &str = "verus";
    const ARG: &str = "proof.rs";
    const PATH_ENV: &str = "PATH";
    const TMPDIR_ENV: &str = "TMPDIR";
    const TIMEOUT_MS: u64 = 30_000;
    const SUCCESS_EXIT: i32 = 0;
    const FAILURE_EXIT: i32 = 1;
    const ZERO_TIMEOUT_MS: u64 = 0;

    #[test]
    fn process_command_plan_is_pure_and_stable() {
        let input = valid_input();

        let first = build_process_command_plan(input.clone()).expect("valid plan");
        let second = build_process_command_plan(input).expect("valid plan");

        assert_eq!(first, second);
        assert_eq!(first.capability, ProcessCapabilityToken::process());
        assert_eq!(first.capability.family.as_str(), PROCESS_EFFECT_FAMILY);
        assert_eq!(first.executable, EXE);
        assert_eq!(first.args, vec![ARG.to_owned()]);
        assert_eq!(first.environment_allowlist, vec![PATH_ENV.to_owned(), TMPDIR_ENV.to_owned()]);
    }

    #[test]
    fn process_command_plan_rejects_broad_or_malformed_effect_inputs() {
        let diagnostics = build_process_command_plan(ProcessCommandPlanInput {
            command_argv: vec!["sh -c verus proof.rs | tee log".to_owned(), String::new()],
            timeout_ms: ZERO_TIMEOUT_MS,
            environment_allowlist: vec!["PATH ".to_owned(), "mixedCase".to_owned(), String::new()],
        })
        .expect_err("malformed plan should fail");

        assert_contains(&diagnostics, "timeout_ms` must be positive");
        assert_contains(&diagnostics, "empty argv segment");
        assert_contains(&diagnostics, "whitespace in executable");
        assert_contains(&diagnostics, "shell metacharacters");
        assert_contains(&diagnostics, "surrounding whitespace");
        assert_contains(&diagnostics, "narrow environment variable name");
        assert_contains(&diagnostics, "empty name");
    }

    #[test]
    fn captured_process_result_classification_has_positive_and_negative_shapes() {
        let plan = build_process_command_plan(valid_input()).expect("valid plan");
        let passed = CapturedProcessResult {
            exit_code: Some(SUCCESS_EXIT),
            timed_out: false,
            stdout: "verified".to_owned(),
            stderr: String::new(),
        };
        let failed = CapturedProcessResult {
            exit_code: Some(FAILURE_EXIT),
            timed_out: false,
            stdout: String::new(),
            stderr: "error".to_owned(),
        };
        let timed_out = CapturedProcessResult {
            exit_code: None,
            timed_out: true,
            stdout: String::new(),
            stderr: String::new(),
        };

        assert_eq!(classify_captured_process_result(&plan, Some(&passed)).status, STATUS_PASSED);
        assert_eq!(classify_captured_process_result(&plan, Some(&failed)).status, STATUS_FAILED);
        assert_eq!(classify_captured_process_result(&plan, Some(&timed_out)).status, STATUS_TIMED_OUT);
        assert_eq!(classify_captured_process_result(&plan, None).status, STATUS_MISSING);
    }

    fn valid_input() -> ProcessCommandPlanInput {
        ProcessCommandPlanInput {
            command_argv: vec![EXE.to_owned(), ARG.to_owned()],
            timeout_ms: TIMEOUT_MS,
            environment_allowlist: vec![TMPDIR_ENV.to_owned(), PATH_ENV.to_owned(), PATH_ENV.to_owned()],
        }
    }

    fn assert_contains(diagnostics: &[String], expected: &str) {
        assert!(
            diagnostics.iter().any(|diagnostic| diagnostic.contains(expected)),
            "expected diagnostics to contain `{expected}`, got {diagnostics:?}"
        );
    }
}
