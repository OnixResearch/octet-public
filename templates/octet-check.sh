#!/bin/sh
# Copy into a consumer workspace (for example as `scripts/octet-check`) and
# mark it executable to make the Octet gate the default-on checked build path.
#
# Usage:
#   scripts/octet-check          # runs the Octet gate, then cargo check
#   scripts/octet-check --lib    # runs the gate, then cargo check --lib
#
# Cargo aliases expand only to Cargo arguments; they cannot run two commands
# or stop between them, so the gate needs this wrapper. The gate runs first
# with the workspace's `[workspace.metadata.octet]` defaults. If it fails
# (exit 1 = error-level findings or new baseline IDs, exit 2 = integration
# failure), the script exits with the gate's status and `cargo check` does not
# run. Arguments are passed to `cargo check` only.
set -eu

cargo octet check
exec cargo check "$@"
