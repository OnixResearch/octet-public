use vstd::prelude::*;
verus! {
pub proof fn reflexive(value: int)
    ensures value == value,
{}
} // verus!
