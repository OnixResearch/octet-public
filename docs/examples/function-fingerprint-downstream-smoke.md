# Function fingerprint downstream smoke

The flake exposes a scratch-consumer smoke for the checked fingerprint docs examples.
Most reviewers should run the aggregate executable-docs gate instead of this focused
check directly:

```bash
nix build .#checks.x86_64-linux.docs-examples --no-link -L
```

Use the focused downstream smoke only when debugging the consumer-path slice:

```bash
nix build .#checks.x86_64-linux.fingerprint-docs-downstream-smoke --no-link -L
```

The check creates a temporary Cargo crate, copies every checked
`docs/examples/function-fingerprint-*/reviewed.rs` fixture into that crate, and
runs the real `cargo-octet fingerprint` binary over the consumer `src/`
directory. It asserts that:

- all seven documented records are discovered outside the Octet checkout;
- `macro_rules!` generated functions are still absent from the record set;
- warning and missing-metadata examples keep their nonzero operator-warning status;
- JSON drift, human `--diff`, JSON `--diff`, update, and repaired-check behavior work on a copied downstream file;
- human and JSON diff modes identify the same mismatched function before repair.

The reusable script is also exposed as:

```bash
nix run .#fingerprint-docs-downstream-smoke
```
