let Contracts = import "../../contracts.ncl" in
let Values = import "../values.ncl" in
let PromotedProofCard = Values.ProofCard & {
  evidence_origin | force = Contracts.constants.ExternalTrellisOrigin,
  evidence_tier | force = Contracts.constants.FormalProofTier,
} in
(
  Values.OctetExport & {
    proof_cards | force = [PromotedProofCard],
  }
)
| Contracts.OctetValenceProvenanceExportShape
| Contracts.OctetValenceProvenanceExport
