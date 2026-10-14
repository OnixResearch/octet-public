use vstd::prelude::*;

verus! {

// Failure-path evidence only: this intentionally unprovable postcondition
// MUST be rejected by real Verus and MUST NOT be treated as a formal proof.
pub proof fn invalid_addition_bound_rejects_false_postcondition(value: nat)
    ensures
        value + 1 == value,
{
}

} // verus!
fn main() {}
