let contracts = import "../../contracts/generated.ncl" in
{
  property_id = "octet.evidence-rails.hegel-property.valid",
  linked_ids = ["r[octet.dynamic_evidence.hegel_property_receipt]"],
  source_scope = "cargo-octet/src/evidence_rails.rs::evaluate",
  runner = {
    crate_version = "0.14.12",
    backend = "native",
  },
  settings = {
    case_budget = 16,
    phases = ["generate", "shrink"],
    derandomized = true,
  },
  generator = {
    family = "octet-evidence-rail-arbitrary",
    content_hash = "b3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  },
  source_hashes = ["b3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"],
} | contracts.HegelPropertyReceiptDetailsV1
