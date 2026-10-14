use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceLabel {
    pub value: String,
}

pub fn evidence_label(value: String) -> EvidenceLabel {
    EvidenceLabel { value }
}
