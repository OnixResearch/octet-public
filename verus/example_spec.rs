use vstd::prelude::*;

verus! {

pub proof fn checked_add_one_preserves_increment(value: nat)
    ensures
        value + 1 >= value,
{
}

} // verus!
fn main() {}
