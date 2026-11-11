//! CLI shell for the Valence Evidence IR.
//!
//! This is the imperative shell: argument parsing, file reads, stdout/stderr,
//! and exit codes. All core logic lives in `valence_core`.

use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

use cap_std::ambient_authority;
use cap_std::fs::Dir;

const VERSION: &str = "0.1.0";
const USAGE: &str = "\
valence — Evidence IR CLI

USAGE:
    valence <COMMAND> [OPTIONS]

COMMANDS:
    check --artifact-root <ROOT> <FILE>  Validate a root-relative Evidence IR artifact
    explain --manifest-root <ROOT> --id <ID> --manifest <FILE>
    stack-provenance build --artifact-root <ROOT> --octet-export <FILE> --binary-hash <B3> --output-root <ROOT> --output <FILE>
    stack-provenance verify --manifest-root <ROOT> --manifest <FILE> --sidecar-root <ROOT> --sidecar <FILE>
    stack-provenance verify-built-binary --binary-root <ROOT> --binary <FILE> --artifact-root <ROOT> --octet-export <FILE> --sidecar-root <ROOT> --sidecar <FILE> --manifest-root <ROOT> --manifest <FILE> [--json]
    stack-provenance graph-report --artifact-root <ROOT> --chain <FILE> [--format json|markdown]
    function-address validate --artifact-root <ROOT> --input <FILE> [--output-root <ROOT> --bundle <FILE>] [--receipt <FILE>] [--graph <FILE>]
    evidence-chain validate --artifact-root <ROOT> --input <FILE>
    version                             Print version

EXIT CODES:
    0   success
    1   validation failure or not found
    2   usage error or I/O failure
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprint!("{USAGE}");
        return ExitCode::from(2);
    }

    match args[1].as_str() {
        "version" | "--version" | "-V" => {
            println!("valence {VERSION}");
            ExitCode::SUCCESS
        }
        "check" => run_check(&args[2..]),
        "explain" => run_explain(&args[2..]),
        "stack-provenance" => run_stack_provenance(&args[2..]),
        "function-address" => run_function_address(&args[2..]),
        "evidence-chain" => run_evidence_chain(&args[2..]),
        "wasm-component" => run_wasm_component(&args[2..]),
        "help" | "--help" | "-h" => {
            print!("{USAGE}");
            ExitCode::SUCCESS
        }
        command => {
            eprintln!("unknown command: {command}");
            eprint!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

fn run_check(args: &[String]) -> ExitCode {
    const CHECK_ARGUMENT_COUNT: usize = 3;
    if args.len() != CHECK_ARGUMENT_COUNT || args[0] != "--artifact-root" {
        eprintln!("usage: valence check --artifact-root <ROOT> <FILE>");
        return ExitCode::from(2);
    }

    let root = PathBuf::from(&args[1]);
    let path = PathBuf::from(&args[2]);
    let json = match cap_read_to_string(CapabilityRootKind::InputArtifact, &root, &path) {
        Ok(json) => json,
        Err(error) => {
            eprintln!("error: cannot read {}: {error}", path.display());
            return ExitCode::from(2);
        }
    };

    let report = valence_core::evidence_ir::check_evidence_artifact(&json);
    if report.valid {
        println!("valid: true");
        println!("issue_count=0");
        print_non_claims(&report.non_claims);
        ExitCode::SUCCESS
    } else {
        println!("valid: false");
        println!("issue_count={}", report.issue_codes.len());
        for code in &report.issue_codes {
            println!("issue={code}");
        }
        print_non_claims(&report.non_claims);
        ExitCode::from(1)
    }
}

fn run_function_address(args: &[String]) -> ExitCode {
    let Some(command) = args.first() else {
        eprintln!("usage: valence function-address validate --input <FILE> [--receipt <FILE>] [--graph <FILE>]");
        return ExitCode::from(2);
    };
    match command.as_str() {
        "validate" => run_function_address_validate(&args[1..]),
        "preserves" => run_function_address_preserves(&args[1..]),
        other => {
            eprintln!("unknown function-address command: {other}");
            ExitCode::from(2)
        }
    }
}

fn run_function_address_preserves(args: &[String]) -> ExitCode {
    const ARGUMENT_COUNT: usize = 4;
    if args.len() != ARGUMENT_COUNT || args[0] != "--artifact-root" || args[2] != "--input" {
        eprintln!("usage: valence function-address preserves --artifact-root <ROOT> --input <FILE>");
        return ExitCode::from(2);
    }
    let json = match cap_read_to_string(CapabilityRootKind::InputArtifact, Path::new(&args[1]), Path::new(&args[3])) {
        Ok(json) => json,
        Err(error) => {
            eprintln!("error: cannot read Preserves receipt: {error}");
            return ExitCode::from(2);
        }
    };
    let receipt: valence_core::function_address_preserves::FunctionAddressPreservesReceipt =
        match serde_json::from_str(&json) {
            Ok(receipt) => receipt,
            Err(error) => {
                eprintln!("error: cannot parse Preserves receipt envelope: {error}");
                return ExitCode::from(2);
            }
        };
    let report = valence_core::function_address_preserves::validate_function_address_preserves(&receipt);
    match serde_json::to_string_pretty(&report) {
        Ok(json) => println!("{json}"),
        Err(error) => {
            eprintln!("error: cannot serialize Preserves report: {error}");
            return ExitCode::from(2);
        }
    }
    if report.valid {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn run_function_address_validate(args: &[String]) -> ExitCode {
    let mut artifact_root: Option<PathBuf> = None;
    let mut output_root: Option<PathBuf> = None;
    let mut input_path: Option<PathBuf> = None;
    let mut bundle_path: Option<PathBuf> = None;
    let mut receipt_path: Option<PathBuf> = None;
    let mut graph_path: Option<PathBuf> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--artifact-root" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --artifact-root requires a value");
                    return ExitCode::from(2);
                }
                artifact_root = Some(PathBuf::from(&args[i]));
            }
            "--output-root" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --output-root requires a value");
                    return ExitCode::from(2);
                }
                output_root = Some(PathBuf::from(&args[i]));
            }
            "--input" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --input requires a value");
                    return ExitCode::from(2);
                }
                input_path = Some(PathBuf::from(&args[i]));
            }
            "--bundle" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --bundle requires a value");
                    return ExitCode::from(2);
                }
                bundle_path = Some(PathBuf::from(&args[i]));
            }
            "--receipt" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --receipt requires a value");
                    return ExitCode::from(2);
                }
                receipt_path = Some(PathBuf::from(&args[i]));
            }
            "--graph" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --graph requires a value");
                    return ExitCode::from(2);
                }
                graph_path = Some(PathBuf::from(&args[i]));
            }
            other => {
                eprintln!("unknown function-address option: {other}");
                return ExitCode::from(2);
            }
        }
        i += 1;
    }
    let Some(artifact_root) = artifact_root else {
        eprintln!("error: --artifact-root is required");
        return ExitCode::from(2);
    };
    let Some(input_path) = input_path else {
        eprintln!("error: --input is required");
        return ExitCode::from(2);
    };
    if (bundle_path.is_some() || receipt_path.is_some() || graph_path.is_some()) && output_root.is_none() {
        eprintln!("error: --output-root is required for publication outputs");
        return ExitCode::from(2);
    }
    let input_json = match cap_read_to_string(CapabilityRootKind::InputArtifact, &artifact_root, &input_path) {
        Ok(json) => json,
        Err(error) => {
            eprintln!("error: cannot read {}: {error}", input_path.display());
            return ExitCode::from(2);
        }
    };
    let input: valence_core::function_address_profile::FunctionAddressEvidenceInput =
        match serde_json::from_str(&input_json) {
            Ok(input) => input,
            Err(error) => {
                eprintln!("error: cannot parse function-address input: {error}");
                return ExitCode::from(2);
            }
        };
    // r[impl valence.evidence_ir.function_address_publication.invalid_no_write]
    let decision = valence_core::function_address_profile::plan_function_address_publication(&input);
    if !decision.valid {
        return print_function_address_decision(&decision, ExitCode::from(1));
    }
    let Some(publication) = decision.publication.as_ref() else {
        eprintln!("error: valid publication decision omitted publication material");
        return ExitCode::from(2);
    };
    let bundle_bytes = match serde_json::to_vec_pretty(&publication.bundle) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("error: cannot serialize function-address bundle: {error}");
            return ExitCode::from(2);
        }
    };
    let receipt_bytes = match serde_json::to_vec_pretty(&publication.compatibility_receipt) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("error: cannot serialize compatibility receipt: {error}");
            return ExitCode::from(2);
        }
    };
    let graph_bytes = match serde_json::to_vec_pretty(&publication.compatibility_graph) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("error: cannot serialize compatibility graph: {error}");
            return ExitCode::from(2);
        }
    };
    let root = output_root.as_deref().unwrap_or_else(|| Path::new(""));
    if let Some(path) = bundle_path
        && let Err(error) = cap_atomic_write(CapabilityRootKind::Output, root, &path, &bundle_bytes)
    {
        eprintln!("error: cannot publish bundle {}: {error}", path.display());
        return ExitCode::from(2);
    }
    if let Err(error) = cap_atomic_write_pair(
        CapabilityRootKind::Output,
        root,
        receipt_path.as_deref().map(|path| (path, receipt_bytes.as_slice())),
        graph_path.as_deref().map(|path| (path, graph_bytes.as_slice())),
    ) {
        eprintln!("error: cannot publish compatibility outputs: {error}");
        return ExitCode::from(2);
    }
    print_function_address_decision(&decision, ExitCode::SUCCESS)
}

fn run_wasm_component(args: &[String]) -> ExitCode {
    const ARGUMENT_COUNT: usize = 5;
    if args.len() != ARGUMENT_COUNT || args[0] != "validate" || args[1] != "--artifact-root" || args[3] != "--input" {
        eprintln!("usage: valence wasm-component validate --artifact-root <ROOT> --input <FILE>");
        return ExitCode::from(2);
    }
    let json = match cap_read_to_string(CapabilityRootKind::InputArtifact, Path::new(&args[2]), Path::new(&args[4])) {
        Ok(json) => json,
        Err(error) => {
            eprintln!("error: cannot read Wasm component graph: {error}");
            return ExitCode::from(2);
        }
    };
    let graph: valence_core::wasm_component_evidence::WasmComponentEvidenceGraph = match serde_json::from_str(&json) {
        Ok(graph) => graph,
        Err(error) => {
            eprintln!("error: cannot parse Wasm component graph: {error}");
            return ExitCode::from(2);
        }
    };
    let report = valence_core::wasm_component_evidence::validate_wasm_component_graph(&graph);
    match serde_json::to_string_pretty(&report) {
        Ok(json) => println!("{json}"),
        Err(error) => {
            eprintln!("error: cannot serialize Wasm report: {error}");
            return ExitCode::from(2);
        }
    }
    if report.valid {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn run_evidence_chain(args: &[String]) -> ExitCode {
    const ARGUMENT_COUNT: usize = 5;
    if args.len() != ARGUMENT_COUNT || args[0] != "validate" || args[1] != "--artifact-root" || args[3] != "--input" {
        eprintln!("usage: valence evidence-chain validate --artifact-root <ROOT> --input <FILE>");
        return ExitCode::from(2);
    }
    let root = Path::new(&args[2]);
    let locator = Path::new(&args[4]);
    let json = match cap_read_to_string(CapabilityRootKind::InputArtifact, root, locator) {
        Ok(json) => json,
        Err(error) => {
            eprintln!("error: cannot read evidence-chain input: {error}");
            return ExitCode::from(2);
        }
    };
    let envelope: valence_core::evidence_chain::EvidenceChainEnvelope = match serde_json::from_str(&json) {
        Ok(envelope) => envelope,
        Err(error) => {
            eprintln!("error: cannot parse evidence-chain input: {error}");
            return ExitCode::from(2);
        }
    };
    let report = valence_core::evidence_chain::validate_evidence_chain(&envelope);
    match serde_json::to_string_pretty(&report) {
        Ok(json) => println!("{json}"),
        Err(error) => {
            eprintln!("error: cannot serialize evidence-chain report: {error}");
            return ExitCode::from(2);
        }
    }
    if report.valid {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn run_stack_provenance(args: &[String]) -> ExitCode {
    let Some(command) = args.first() else {
        eprintln!("usage: valence stack-provenance <build|verify> ...");
        return ExitCode::from(2);
    };
    match command.as_str() {
        "build" => run_stack_provenance_build(&args[1..]),
        "verify" => run_stack_provenance_verify(&args[1..]),
        "verify-built-binary" => run_stack_provenance_verify_built_binary(&args[1..]),
        "graph-report" => run_stack_provenance_graph_report(&args[1..]),
        other => {
            eprintln!("unknown stack-provenance command: {other}");
            ExitCode::from(2)
        }
    }
}

fn run_stack_provenance_build(args: &[String]) -> ExitCode {
    let mut artifact_root: Option<PathBuf> = None;
    let mut output_root: Option<PathBuf> = None;
    let mut octet_export_path: Option<PathBuf> = None;
    let mut binary_hash: Option<String> = None;
    let mut output_path: Option<PathBuf> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--artifact-root" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --artifact-root requires a value");
                    return ExitCode::from(2);
                }
                artifact_root = Some(PathBuf::from(&args[i]));
            }
            "--output-root" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --output-root requires a value");
                    return ExitCode::from(2);
                }
                output_root = Some(PathBuf::from(&args[i]));
            }
            "--octet-export" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --octet-export requires a value");
                    return ExitCode::from(2);
                }
                octet_export_path = Some(PathBuf::from(&args[i]));
            }
            "--binary-hash" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --binary-hash requires a value");
                    return ExitCode::from(2);
                }
                binary_hash = Some(args[i].clone());
            }
            "--output" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --output requires a value");
                    return ExitCode::from(2);
                }
                output_path = Some(PathBuf::from(&args[i]));
            }
            other => {
                eprintln!("unknown option: {other}");
                return ExitCode::from(2);
            }
        }
        i += 1;
    }

    let Some(artifact_root) = artifact_root else {
        eprintln!("error: --artifact-root is required");
        return ExitCode::from(2);
    };
    let Some(output_root) = output_root else {
        eprintln!("error: --output-root is required");
        return ExitCode::from(2);
    };
    let Some(octet_export_path) = octet_export_path else {
        eprintln!("error: --octet-export is required");
        return ExitCode::from(2);
    };
    let Some(binary_hash) = binary_hash else {
        eprintln!("error: --binary-hash is required");
        return ExitCode::from(2);
    };
    let Some(output_path) = output_path else {
        eprintln!("error: --output is required");
        return ExitCode::from(2);
    };

    let octet_export_json =
        match cap_read_to_string(CapabilityRootKind::InputArtifact, &artifact_root, &octet_export_path) {
            Ok(json) => json,
            Err(error) => {
                eprintln!("error: cannot read {}: {error}", octet_export_path.display());
                return ExitCode::from(2);
            }
        };
    let sidecar_json = match valence_core::stack_provenance_adapter::build_sidecar_json_from_octet_export_json(
        &octet_export_json,
        &binary_hash,
    ) {
        Ok(sidecar_json) => sidecar_json,
        Err(report) => {
            print_stack_provenance_report(&report);
            return ExitCode::from(1);
        }
    };
    if let Err(error) = cap_write(CapabilityRootKind::Output, &output_root, &output_path, sidecar_json.as_bytes()) {
        eprintln!("error: cannot write {}: {error}", output_path.display());
        return ExitCode::from(2);
    }
    println!("valid: true");
    println!("wrote: {}", output_path.display());
    ExitCode::SUCCESS
}

fn run_stack_provenance_verify(args: &[String]) -> ExitCode {
    let mut manifest_root: Option<PathBuf> = None;
    let mut sidecar_root: Option<PathBuf> = None;
    let mut manifest_path: Option<PathBuf> = None;
    let mut sidecar_path: Option<PathBuf> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--manifest-root" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --manifest-root requires a value");
                    return ExitCode::from(2);
                }
                manifest_root = Some(PathBuf::from(&args[i]));
            }
            "--sidecar-root" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --sidecar-root requires a value");
                    return ExitCode::from(2);
                }
                sidecar_root = Some(PathBuf::from(&args[i]));
            }
            "--manifest" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --manifest requires a value");
                    return ExitCode::from(2);
                }
                manifest_path = Some(PathBuf::from(&args[i]));
            }
            "--sidecar" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --sidecar requires a value");
                    return ExitCode::from(2);
                }
                sidecar_path = Some(PathBuf::from(&args[i]));
            }
            other => {
                eprintln!("unknown option: {other}");
                return ExitCode::from(2);
            }
        }
        i += 1;
    }

    let Some(manifest_root) = manifest_root else {
        eprintln!("error: --manifest-root is required");
        return ExitCode::from(2);
    };
    let Some(sidecar_root) = sidecar_root else {
        eprintln!("error: --sidecar-root is required");
        return ExitCode::from(2);
    };
    let Some(manifest_path) = manifest_path else {
        eprintln!("error: --manifest is required");
        return ExitCode::from(2);
    };
    let Some(sidecar_path) = sidecar_path else {
        eprintln!("error: --sidecar is required");
        return ExitCode::from(2);
    };
    let manifest_json = match cap_read_to_string(CapabilityRootKind::Manifest, &manifest_root, &manifest_path) {
        Ok(json) => json,
        Err(error) => {
            eprintln!("error: cannot read {}: {error}", manifest_path.display());
            return ExitCode::from(2);
        }
    };
    let sidecar_json = match cap_read_to_string(CapabilityRootKind::Sidecar, &sidecar_root, &sidecar_path) {
        Ok(json) => json,
        Err(error) => {
            eprintln!("error: cannot read {}: {error}", sidecar_path.display());
            return ExitCode::from(2);
        }
    };
    let report =
        valence_core::stack_provenance_adapter::verify_mantle_external_evidence_sidecar(&manifest_json, &sidecar_json);
    print_stack_provenance_report(&report);
    if report.valid {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn run_stack_provenance_graph_report(args: &[String]) -> ExitCode {
    // r[impl valence.stack_provenance_adapter.graph_report]
    let mut artifact_root: Option<PathBuf> = None;
    let mut chain_path: Option<PathBuf> = None;
    let mut format = "json".to_owned();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--artifact-root" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --artifact-root requires a value");
                    return ExitCode::from(2);
                }
                artifact_root = Some(PathBuf::from(&args[i]));
            }
            "--chain" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --chain requires a value");
                    return ExitCode::from(2);
                }
                chain_path = Some(PathBuf::from(&args[i]));
            }
            "--format" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --format requires a value");
                    return ExitCode::from(2);
                }
                format = args[i].clone();
            }
            other => {
                eprintln!("unknown option: {other}");
                return ExitCode::from(2);
            }
        }
        i += 1;
    }

    let Some(artifact_root) = artifact_root else {
        eprintln!("error: --artifact-root is required");
        return ExitCode::from(2);
    };
    let Some(chain_path) = chain_path else {
        eprintln!("error: --chain is required");
        return ExitCode::from(2);
    };
    let chain_json = match cap_read_to_string(CapabilityRootKind::InputArtifact, &artifact_root, &chain_path) {
        Ok(json) => json,
        Err(error) => {
            eprintln!("error: cannot read {}: {error}", chain_path.display());
            return ExitCode::from(2);
        }
    };
    let report = valence_core::stack_provenance_adapter::stack_provenance_graph_report_from_json(&chain_json);
    match format.as_str() {
        "json" => match serde_json::to_string_pretty(&report) {
            Ok(json) => println!("{json}"),
            Err(error) => {
                eprintln!("error: cannot serialize graph report: {error}");
                return ExitCode::from(2);
            }
        },
        "markdown" => print!("{}", render_stack_provenance_graph_markdown(&report)),
        other => {
            eprintln!("unsupported graph-report format: {other}");
            return ExitCode::from(2);
        }
    }
    if report.valid {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn run_stack_provenance_verify_built_binary(args: &[String]) -> ExitCode {
    let mut binary_root: Option<PathBuf> = None;
    let mut artifact_root: Option<PathBuf> = None;
    let mut sidecar_root: Option<PathBuf> = None;
    let mut manifest_root: Option<PathBuf> = None;
    let mut binary_path: Option<PathBuf> = None;
    let mut octet_export_path: Option<PathBuf> = None;
    let mut sidecar_path: Option<PathBuf> = None;
    let mut manifest_path: Option<PathBuf> = None;
    let mut json_output = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--binary-root" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --binary-root requires a value");
                    return ExitCode::from(2);
                }
                binary_root = Some(PathBuf::from(&args[i]));
            }
            "--artifact-root" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --artifact-root requires a value");
                    return ExitCode::from(2);
                }
                artifact_root = Some(PathBuf::from(&args[i]));
            }
            "--sidecar-root" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --sidecar-root requires a value");
                    return ExitCode::from(2);
                }
                sidecar_root = Some(PathBuf::from(&args[i]));
            }
            "--manifest-root" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --manifest-root requires a value");
                    return ExitCode::from(2);
                }
                manifest_root = Some(PathBuf::from(&args[i]));
            }
            "--binary" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --binary requires a value");
                    return ExitCode::from(2);
                }
                binary_path = Some(PathBuf::from(&args[i]));
            }
            "--octet-export" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --octet-export requires a value");
                    return ExitCode::from(2);
                }
                octet_export_path = Some(PathBuf::from(&args[i]));
            }
            "--sidecar" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --sidecar requires a value");
                    return ExitCode::from(2);
                }
                sidecar_path = Some(PathBuf::from(&args[i]));
            }
            "--manifest" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --manifest requires a value");
                    return ExitCode::from(2);
                }
                manifest_path = Some(PathBuf::from(&args[i]));
            }
            "--json" => json_output = true,
            other => {
                eprintln!("unknown option: {other}");
                return ExitCode::from(2);
            }
        }
        i += 1;
    }

    let Some(binary_root) = binary_root else {
        eprintln!("error: --binary-root is required");
        return ExitCode::from(2);
    };
    let Some(artifact_root) = artifact_root else {
        eprintln!("error: --artifact-root is required");
        return ExitCode::from(2);
    };
    let Some(sidecar_root) = sidecar_root else {
        eprintln!("error: --sidecar-root is required");
        return ExitCode::from(2);
    };
    let Some(manifest_root) = manifest_root else {
        eprintln!("error: --manifest-root is required");
        return ExitCode::from(2);
    };
    let Some(binary_path) = binary_path else {
        eprintln!("error: --binary is required");
        return ExitCode::from(2);
    };
    let Some(octet_export_path) = octet_export_path else {
        eprintln!("error: --octet-export is required");
        return ExitCode::from(2);
    };
    let Some(sidecar_path) = sidecar_path else {
        eprintln!("error: --sidecar is required");
        return ExitCode::from(2);
    };
    let Some(manifest_path) = manifest_path else {
        eprintln!("error: --manifest is required");
        return ExitCode::from(2);
    };

    let binary_bytes = match cap_read(CapabilityRootKind::Binary, &binary_root, &binary_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("error: cannot read {}: {error}", binary_path.display());
            return ExitCode::from(2);
        }
    };
    let octet_export_json =
        match cap_read_to_string(CapabilityRootKind::InputArtifact, &artifact_root, &octet_export_path) {
            Ok(json) => json,
            Err(error) => {
                eprintln!("error: cannot read {}: {error}", octet_export_path.display());
                return ExitCode::from(2);
            }
        };
    let sidecar_json = match cap_read_to_string(CapabilityRootKind::Sidecar, &sidecar_root, &sidecar_path) {
        Ok(json) => json,
        Err(error) => {
            eprintln!("error: cannot read {}: {error}", sidecar_path.display());
            return ExitCode::from(2);
        }
    };
    let manifest_json = match cap_read_to_string(CapabilityRootKind::Manifest, &manifest_root, &manifest_path) {
        Ok(json) => json,
        Err(error) => {
            eprintln!("error: cannot read {}: {error}", manifest_path.display());
            return ExitCode::from(2);
        }
    };

    let report = valence_core::stack_provenance_adapter::verify_built_binary_thread_json(
        &binary_bytes,
        &octet_export_json,
        &sidecar_json,
        &manifest_json,
    );
    if json_output {
        match serde_json::to_string_pretty(&report) {
            Ok(json) => println!("{json}"),
            Err(error) => {
                eprintln!("error: cannot serialize report: {error}");
                return ExitCode::from(2);
            }
        }
    } else {
        print_built_binary_thread_report(&report);
    }
    if report.valid {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CapabilityRootKind {
    InputArtifact,
    Manifest,
    Sidecar,
    Binary,
    Output,
}

impl CapabilityRootKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::InputArtifact => "input artifact",
            Self::Manifest => "manifest",
            Self::Sidecar => "sidecar",
            Self::Binary => "binary",
            Self::Output => "output",
        }
    }

    fn core_role(self) -> valence_core::capability_plan::CapabilityRole {
        use valence_core::capability_plan::CapabilityRole;
        match self {
            Self::InputArtifact => CapabilityRole::Artifact,
            Self::Manifest => CapabilityRole::Manifest,
            Self::Sidecar => CapabilityRole::Sidecar,
            Self::Binary => CapabilityRole::Binary,
            Self::Output => CapabilityRole::Output,
        }
    }
}

fn cap_read_to_string(kind: CapabilityRootKind, root_path: &Path, locator: &Path) -> Result<String, String> {
    let (root, relative) = open_capability_root(kind, root_path, locator, false)?;
    ensure_existing_path_confined(&root, &relative)?;
    root.read_to_string(&relative).map_err(|error| format!("{} read failed: {error}", kind.as_str()))
}

fn cap_read(kind: CapabilityRootKind, root_path: &Path, locator: &Path) -> Result<Vec<u8>, String> {
    let (root, relative) = open_capability_root(kind, root_path, locator, false)?;
    ensure_existing_path_confined(&root, &relative)?;
    root.read(&relative).map_err(|error| format!("{} read failed: {error}", kind.as_str()))
}

fn cap_write(kind: CapabilityRootKind, root_path: &Path, locator: &Path, bytes: &[u8]) -> Result<(), String> {
    let (root, relative) = open_capability_root(kind, root_path, locator, true)?;
    ensure_output_parent_confined(&root, &relative)?;
    root.write(&relative, bytes).map_err(|error| format!("{} write failed: {error}", kind.as_str()))
}

fn cap_atomic_write(kind: CapabilityRootKind, root_path: &Path, locator: &Path, bytes: &[u8]) -> Result<(), String> {
    // r[impl valence.evidence_ir.function_address_publication.atomic_bundle]
    let (root, relative) = open_capability_root(kind, root_path, locator, true)?;
    let staging = stage_atomic_write(&root, &relative, bytes)?;
    root.rename(&staging, &root, &relative)
        .map_err(|error| format!("{} atomic replace failed: {error}", kind.as_str()))
}

fn cap_atomic_write_pair(
    kind: CapabilityRootKind,
    root_path: &Path,
    first: Option<(&Path, &[u8])>,
    second: Option<(&Path, &[u8])>,
) -> Result<(), String> {
    // r[impl valence.evidence_ir.function_address_publication.compatibility_outputs]
    let Some((anchor_locator, anchor_bytes)) = first.or(second) else {
        return Ok(());
    };
    if first.is_none() || second.is_none() {
        return cap_atomic_write(kind, root_path, anchor_locator, anchor_bytes);
    }
    let (first_locator, first_bytes) = first.expect("checked first compatibility output");
    let (second_locator, second_bytes) = second.expect("checked second compatibility output");
    let (root, first_relative) = open_capability_root(kind, root_path, first_locator, true)?;
    let (_, second_relative) = open_capability_root(kind, root_path, second_locator, true)?;
    let first_staging = stage_atomic_write(&root, &first_relative, first_bytes)?;
    let second_staging = match stage_atomic_write(&root, &second_relative, second_bytes) {
        Ok(staging) => staging,
        Err(error) => {
            let _ = root.remove_file(&first_staging);
            return Err(error);
        }
    };
    root.rename(&first_staging, &root, &first_relative)
        .map_err(|error| format!("first compatibility replace failed: {error}"))?;
    root.rename(&second_staging, &root, &second_relative)
        .map_err(|error| format!("second compatibility replace failed; outputs are not cross-file atomic: {error}"))
}

fn stage_atomic_write(root: &Dir, relative: &Path, bytes: &[u8]) -> Result<PathBuf, String> {
    ensure_output_parent_confined(root, relative)?;
    let file_name = relative.file_name().ok_or_else(|| "output locator must name a file".to_owned())?;
    let staging_name = format!(".{}.valence-stage", file_name.to_string_lossy());
    let staging = relative.with_file_name(staging_name);
    let _ = root.remove_file(&staging);
    let mut options = cap_std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    let mut file =
        root.open_with(&staging, &options).map_err(|error| format!("output staging open failed: {error}"))?;
    file.write_all(bytes).map_err(|error| format!("output staging write failed: {error}"))?;
    file.sync_all().map_err(|error| format!("output staging flush failed: {error}"))?;
    Ok(staging)
}

fn print_function_address_decision(
    decision: &valence_core::function_address_profile::FunctionAddressPublicationDecision,
    exit: ExitCode,
) -> ExitCode {
    match serde_json::to_string_pretty(decision) {
        Ok(json) => {
            println!("{json}");
            exit
        }
        Err(error) => {
            eprintln!("error: cannot serialize function-address decision: {error}");
            ExitCode::from(2)
        }
    }
}

fn open_capability_root(
    kind: CapabilityRootKind,
    root_path: &Path,
    locator: &Path,
    write: bool,
) -> Result<(Dir, PathBuf), String> {
    use valence_core::capability_plan::CapabilityAccessPlan;
    use valence_core::capability_plan::CapabilityOperation;

    let plan = CapabilityAccessPlan {
        role: kind.core_role(),
        operation: if write {
            CapabilityOperation::Write
        } else {
            CapabilityOperation::Read
        },
        root_declaration: root_path.to_string_lossy().into_owned(),
        locator: locator.to_string_lossy().into_owned(),
    };
    let issues = valence_core::capability_plan::validate_capability_access_plan(&plan);
    if let Some(issue) = issues.first() {
        return Err(format!("{}:{}", issue.field, issue.code));
    }
    let root = Dir::open_ambient_dir(root_path, ambient_authority())
        .map_err(|error| format!("{} root {} could not be opened: {error}", kind.as_str(), root_path.display()))?;
    Ok((root, locator.to_path_buf()))
}

fn ensure_existing_path_confined(root: &Dir, relative: &Path) -> Result<(), String> {
    let canonical = root
        .canonicalize(relative)
        .map_err(|error| format!("capability path is unavailable or escapes root: {error}"))?;
    if canonical.is_absolute()
        || canonical.components().any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err("capability path escapes declared root".to_owned());
    }
    Ok(())
}

fn ensure_output_parent_confined(root: &Dir, relative: &Path) -> Result<(), String> {
    if let Some(parent) = relative.parent()
        && !parent.as_os_str().is_empty()
    {
        ensure_existing_path_confined(root, parent)?;
    }
    Ok(())
}

fn print_non_claims(non_claims: &[String]) {
    println!("non_claim_count={}", non_claims.len());
    for claim in non_claims {
        println!("non_claim={claim}");
    }
}

fn print_stack_provenance_report(report: &valence_core::stack_provenance_adapter::StackProvenanceReport) {
    println!("valid: {}", report.valid);
    println!("issue_count={}", report.issue_codes.len());
    for code in &report.issue_codes {
        println!("issue={code}");
    }
}

fn render_stack_provenance_graph_markdown(
    report: &valence_core::stack_provenance_adapter::StackProvenanceGraphReport,
) -> String {
    // r[impl valence.stack_provenance_adapter.graph_report]
    let mut output = String::new();
    output.push_str("# Stack provenance graph report\n\n");
    output.push_str(&format!("- valid: {}\n", report.valid));
    output.push_str(&format!("- artifacts: {}\n", report.artifact_count));
    output.push_str(&format!("- edges: {}\n", report.edge_count));
    output.push_str(&format!("- report_hash: {}\n", report.report_hash));
    output.push_str(&format!("- supported_claim: {}\n", report.supported_claim));
    output.push_str("\n## Diagnostics\n");
    if report.diagnostics.is_empty() {
        output.push_str("- none\n");
    } else {
        for diagnostic in &report.diagnostics {
            output.push_str(&format!("- {diagnostic}\n"));
        }
    }
    output.push_str("\n## Trust boundaries\n");
    for boundary in &report.trust_boundaries {
        output.push_str(&format!("- {boundary}\n"));
    }
    output.push_str("\n## Manual-review nodes\n");
    if report.manual_nodes.is_empty() {
        output.push_str("- none\n");
    } else {
        for node in &report.manual_nodes {
            output.push_str(&format!("- {node}\n"));
        }
    }
    output.push_str("\n## Non-claims\n");
    for non_claim in &report.non_claims {
        output.push_str(&format!("- {non_claim}\n"));
    }
    output
}

fn print_built_binary_thread_report(report: &valence_core::stack_provenance_adapter::BuiltBinaryThreadReport) {
    println!("valid: {}", report.valid);
    println!("issue_count={}", report.issue_codes.len());
    println!("binary_hash={}", report.binary_hash);
    println!("sidecar_digest_blake3={}", report.sidecar_digest_blake3);
    println!("preserves_content_hash={}", report.preserves_content_hash);
    for code in &report.issue_codes {
        println!("issue={code}");
    }
    for non_claim in &report.non_claims {
        println!("non_claim={non_claim}");
    }
}

fn run_explain(args: &[String]) -> ExitCode {
    let mut source_id: Option<String> = None;
    let mut manifest_root: Option<PathBuf> = None;
    let mut manifest_path: Option<PathBuf> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--manifest-root" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --manifest-root requires a value");
                    return ExitCode::from(2);
                }
                manifest_root = Some(PathBuf::from(&args[i]));
            }
            "--id" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --id requires a value");
                    return ExitCode::from(2);
                }
                source_id = Some(args[i].clone());
            }
            "--manifest" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("error: --manifest requires a value");
                    return ExitCode::from(2);
                }
                manifest_path = Some(PathBuf::from(&args[i]));
            }
            other => {
                eprintln!("unknown option: {other}");
                return ExitCode::from(2);
            }
        }
        i += 1;
    }

    let Some(source_id) = source_id else {
        eprintln!("error: --id is required");
        return ExitCode::from(2);
    };
    let Some(manifest_root) = manifest_root else {
        eprintln!("error: --manifest-root is required");
        return ExitCode::from(2);
    };
    let Some(manifest_path) = manifest_path else {
        eprintln!("error: --manifest is required");
        return ExitCode::from(2);
    };

    let manifest_json = match cap_read_to_string(CapabilityRootKind::Manifest, &manifest_root, &manifest_path) {
        Ok(json) => json,
        Err(error) => {
            eprintln!("error: cannot read {}: {error}", manifest_path.display());
            return ExitCode::from(2);
        }
    };

    match valence_core::evidence_ir::explain_span(&manifest_json, &source_id) {
        valence_core::evidence_ir::ExplainSpanResult::Found(span) => {
            println!("source_id: {}", span.source_id);
            println!("source_path: {}", span.source_path);
            println!("start_byte: {}", span.start_byte);
            println!("end_byte: {}", span.end_byte);
            println!("start_line: {}", span.start_line);
            println!("start_column: {}", span.start_column);
            println!("end_line: {}", span.end_line);
            println!("end_column: {}", span.end_column);
            if let Some(label) = &span.label {
                println!("label: {label}");
            }
            ExitCode::SUCCESS
        }
        valence_core::evidence_ir::ExplainSpanResult::NotFound => {
            println!("not_found: {source_id}");
            ExitCode::from(1)
        }
        valence_core::evidence_ir::ExplainSpanResult::InvalidManifest(issues) => {
            for issue in issues {
                eprintln!("invalid_manifest: {}:{}", issue.field, issue.code);
            }
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_command_prints_version() {
        assert!(VERSION.contains("0.1."));
    }

    #[test]
    fn usage_documentation_is_stable() {
        assert!(USAGE.contains("check --artifact-root <ROOT> <FILE>"));
        assert!(USAGE.contains("explain --manifest-root <ROOT>"));
        assert!(USAGE.contains("stack-provenance build"));
        assert!(USAGE.contains("stack-provenance verify"));
        assert!(USAGE.contains("stack-provenance verify-built-binary"));
        assert!(USAGE.contains("version"));
        assert!(USAGE.contains("EXIT CODES"));
    }

    #[test]
    fn capability_roots_read_and_write_in_root() {
        // r[verify valence.stack_provenance_adapter.cap_std_cli_boundary.tests.positive]
        let root = temp_capability_dir("positive");
        let input = Path::new("input.json");
        let output = Path::new("output.json");
        std::fs::write(root.join(input), "{\"valid\":true}").expect("write input fixture");

        let read = cap_read_to_string(CapabilityRootKind::InputArtifact, &root, input).expect("read through cap root");
        cap_write(CapabilityRootKind::Output, &root, output, read.as_bytes()).expect("write through cap root");

        assert_eq!(read, "{\"valid\":true}");
        assert_eq!(std::fs::read_to_string(root.join(output)).expect("read output fixture"), read);
    }

    #[test]
    fn missing_capability_root_fails_before_read() {
        // r[verify valence.stack_provenance_adapter.cap_std_cli_boundary.tests.negative]
        let root = temp_capability_dir("missing-root");
        let missing_root = root.join("absent");

        let error = cap_read_to_string(CapabilityRootKind::Manifest, &missing_root, Path::new("input.json"))
            .expect_err("missing root should fail");

        assert!(error.contains("root"));
    }

    #[cfg(unix)]
    #[test]
    fn symlink_escape_is_rejected() {
        // r[verify valence.stack_provenance_adapter.cap_std_cli_boundary.tests.negative]
        let root = temp_capability_dir("symlink-root");
        let outside = temp_capability_dir("symlink-outside");
        let outside_file = outside.join("secret.txt");
        let link = root.join("secret-link.txt");
        std::fs::write(&outside_file, "secret").expect("write outside target");
        std::os::unix::fs::symlink(&outside_file, &link).expect("create symlink escape fixture");

        let error = cap_read_to_string(CapabilityRootKind::InputArtifact, &root, Path::new("secret-link.txt"))
            .expect_err("symlink escape should fail");

        assert!(error.contains("escapes root") || error.contains("absolute"));
    }

    fn temp_capability_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("valence-cap-std-{label}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create temp capability dir");
        dir
    }
}
