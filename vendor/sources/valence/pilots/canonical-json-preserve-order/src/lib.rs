//! Downstream feature-unification fixture for canonical JSON identity.

#[cfg(test)]
mod tests {
    use serde_json::Value;
    use valence_core::shared_identity::canonical_json_bytes;
    use valence_core::shared_identity::canonical_json_hash;

    const CANONICAL_BYTES: &[u8] = br#"{"a":{"m":false,"z":true},"b":[2,1]}"#;
    const CANONICAL_HASH: &str = "05496484bcb1ab5cca6ab4353f23b1b4c4f77db103791f9bd52390908926c72d";

    #[test]
    fn preserve_order_feature_cannot_change_canonical_identity() {
        // r[verify valence.evidence_ir.complete_envelope.feature_matrix]
        let first: Value = serde_json::from_str(r#"{"b":[2,1],"a":{"z":true,"m":false}}"#).expect("first value");
        let second: Value = serde_json::from_str(r#"{"a":{"m":false,"z":true},"b":[2,1]}"#).expect("second value");

        assert_eq!(canonical_json_bytes(&first).expect("canonical bytes"), CANONICAL_BYTES);
        assert_eq!(canonical_json_bytes(&second).expect("canonical bytes"), CANONICAL_BYTES);
        assert_eq!(canonical_json_hash(&first).expect("canonical hash"), CANONICAL_HASH);
        assert_eq!(canonical_json_hash(&second).expect("canonical hash"), CANONICAL_HASH);
    }
}
