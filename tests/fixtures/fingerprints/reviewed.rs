// octet-fingerprint: b3:a51dfd30275003ecdbd5b2f5d2b6b863da41a88dca08b94b15e038739cdefd48; review=safety-critical; evidence=openspec/changes/archive/2026-05-05-add-function-fingerprints/evidence/check-matching.txt
fn checked_transition(input: u32) -> u32 {
    input + 1
}

// octet-fingerprint: b3:7ac614cc7e388f63544631ae86a15f42ecbbc5d1cc099912b726d6774b79d703; review=safety-critical; evidence=openspec/changes/archive/2026-05-05-add-function-fingerprints/evidence/macro-explicit-allowance.txt; macro=allow
fn macro_reviewed(input: u32) -> String {
    format!("{input}")
}
