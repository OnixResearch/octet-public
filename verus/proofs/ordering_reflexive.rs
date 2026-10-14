use vstd::prelude::*;

verus! {

pub proof fn reflexive_order_holds(value: nat)
    ensures
        value <= value,
{
}

} // verus!
fn main() {}
