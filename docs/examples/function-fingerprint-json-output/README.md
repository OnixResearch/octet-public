# Function fingerprint JSON output example

This example documents script-facing `--output-format=json` behavior for
`cargo octet fingerprint`. It records update, clean check, drift check,
accepted update, and final clean-check JSON payloads so automation can rely on
exact fields for mode, per-record status, review metadata, evidence metadata,
macro flags, and hash values.

The executable regression reruns the same workflow in a scratch directory,
parses each JSON payload the CLI prints, requires that payload to appear verbatim
in the transcript, and compares the accepted source to the checked-in
`reviewed.rs` fixture.
