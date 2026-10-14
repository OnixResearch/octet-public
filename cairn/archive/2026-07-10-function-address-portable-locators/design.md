## Design

Octet normalizes every checked source path against the detected workspace root before storing it in a function-address report or baseline key. Absolute paths may still be accepted as operator input paths, but emitted artifacts use repo-relative members.

### Decisions

1. **Workspace-relative storage.** Source envelopes and function records store paths relative to the Cargo workspace root.
2. **Locator separates path and function name.** Locators are derived from normalized relative path plus function display name; host roots are excluded.
3. **Compatibility is bounded.** Existing absolute-path baselines can be recognized when they resolve under the current workspace root, then rewritten in the next observe export.
4. **No path trust claim.** Portable locators prove identity/linkage only and do not prove source provenance or checkout cleanliness.

### Validation shape

Positive tests run the same fixture under two different temporary roots and compare emitted locators. Negative tests cover absolute paths outside the workspace, `..` escapes, duplicated normalized locators, and baselines with stale source identity.
