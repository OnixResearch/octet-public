## Why

Kamacite and Valence need byte spans to preserve source-envelope linkage. The smoke adapter had to recompute spans from Octet's line and column fields, duplicating parser assumptions and creating drift risk.

## What Changes

- Add start/end byte offsets to Octet function-address records.
- Optionally include end line/end column for display while keeping byte offsets authoritative.
- Validate that spans point into the matching source envelope and cover the hashed function body.
- Add positive and negative fixtures for span integrity.

## Impact

Downstream receipts can consume Octet spans directly instead of reconstructing them, making the cross-repo path simpler and more reliable.
