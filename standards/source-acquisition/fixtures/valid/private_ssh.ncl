let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-source-acquisition-receipt/v1",
  evidence_tier = "policy_source_validation",
  rows = [
    {
      label = "private-stack",
      kind = "private_ssh_git",
      locator = "git+ssh://git@github.com/OnixResearch/private-stack",
      revision = "private-rev",
      lock_identity = "b3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      observed_lock_identity = "b3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      credential_boundary = "requires OnixResearch deploy key; not public release source",
      release_eligible = false,
      local_override_policy = "",
      non_claims = ["source acquisition records source identity and policy posture only; it is not dependency audit, vulnerability proof, or formal proof"],
    },
  ],
  parent_artifacts = ["artifacts/source-boundary.json"],
} | contracts.SourceAcquisitionReceiptV1
