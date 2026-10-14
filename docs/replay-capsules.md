# Replay capsules

Octet replay capsules are content-addressed receipts for `cargo octet check`
and `cargo octet fingerprint` evidence. They preserve enough normalized data to
inspect and replay a run without replacing JSON, SARIF, baseline, or fingerprint
outputs.

## Capture

Capture a prepared check manifest:

```sh
cargo octet roadmap capsule capture --input check-capsule-input.json --capsule check-capsule.json
```

Capture a prepared fingerprint manifest the same way:

```sh
cargo octet roadmap capsule capture --input fingerprint-capsule-input.json --capsule fingerprint-capsule.json
```

The input manifest records `command_argv`, `run_kind`, `octet_binary_identity`,
`config_hash`, `input_hashes`, `output_hashes`, and `expected_exit_code`. Local
paths, timestamps, usernames, and secret-like fields are normalized away before
hashing.

## Inspect and replay

Inspect a capsule:

```sh
cargo octet roadmap capsule inspect --capsule check-capsule.json
```

Replay against observed run metadata:

```sh
cargo octet roadmap capsule replay --capsule check-capsule.json --observed observed-check.json
```

Replay accepts exact matches. It also accepts expected failures when the capsule
records a non-zero `expected_exit_code` and the observed run matches that exit
code. Any command, input, config, binary identity, output hash, or exit-code drift
is reported as `drifted` and fails closed.

## Limits

A replay capsule proves identity of captured evidence and replay inputs. It does
not prove that a finding is correct, safe, verified, or semantically complete.
Operators should treat capsules as reproducibility receipts around Octet
outputs, not as a language runtime or proof system.
