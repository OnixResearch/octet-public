## Why

The smoke artifact contained absolute `/tmp/...` paths in source envelopes, function records, locators, baselines, and downstream graph rows. Absolute paths make receipts non-portable and make reviewed baselines sensitive to checkout location.

## What Changes

- Emit repo-relative, slash-normalized source paths for function-address records.
- Define a stable locator format that does not include host-specific path prefixes.
- Preserve source identity separately from display path and checkout root.
- Add migration tests for old absolute-path baselines where safe.

## Impact

Function-address receipts can move between Octet, Kamacite, Valence, Mantle, and Cairn without rewriting locators or graph rows for each workspace path.
