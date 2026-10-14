let contracts = import "../../contracts.ncl" in
let PolicyRegistry = contracts.PolicyRegistry in
let PolicyRegistryRules = contracts.PolicyRegistryRules in
{ schema_version = "fixture", checker_ids = ["registry"], seed_manifests = [{ name = "bad", source_path = "../verified-logic/", source_commit = "fixture", role = "proof-seed", paths = ["src/lib.rs"], provenance = "fixture", license = "fixture", content_hash = "fixture", proof_relevant = true, local_acceptance = [] }], policies = [] } | PolicyRegistry | PolicyRegistryRules
