# Preserves receipt export

<!-- r[verify octet.evidence_provenance.preserves_receipt_export.docs] -->
Preserves receipt export is an optional adapter over already verified Octet artifacts. Octet source receipts, artifact verification, rails, roles, consumer profiles, and non-claims remain authoritative; the Preserves envelope carries canonical identity/linkage for consumers.

A valid export row records:

- source artifact directory and source receipt digest;
- proof that Octet artifact verification already passed;
- Octet rail and evidence role;
- Preserves schema label and canonical envelope digest;
- consumer profile;
- non-claims.

The required non-claim is: Preserves export is an adapter over verified Octet artifacts; Octet source receipts remain authoritative. A Preserves envelope does not prove release eligibility, formal-proof status, source-of-truth ownership, whole-program correctness, or consumer behavior.
