let Contracts = import "../../contracts.ncl" in
let Values = import "../values.ncl" in
(
  Values.MantleManifest & {
    external_evidence | force = [{
      role = "unrelated-release-note",
      schema = Contracts.constants.StackSidecarSchema,
      relative_path = "external-evidence/stack-provenance.json",
      digest_blake3 = Values.SidecarDigest,
      claim_scope = Contracts.constants.ClaimScope,
      non_claims = [Contracts.constants.StackNonClaim],
    }],
  }
)
| Contracts.MantleReleaseManifestViewShape
| Contracts.MantleReleaseManifestView
