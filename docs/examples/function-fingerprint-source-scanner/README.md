# Function fingerprint source-scanner example

This example documents the intentional source-scanner boundary for
`cargo octet fingerprint`.

`macro_rules!` defines `generated_boundary`, but the fingerprint command does not
analyze post-expansion generated functions. The reviewed boundary is the explicit
source wrapper `explicit_generated_boundary`, so the checked transcript contains
one record for that wrapper and no record for the generated function.

Use this pattern when macro-generated code needs review evidence: put a small,
explicit source function around the generated call and fingerprint that wrapper.
