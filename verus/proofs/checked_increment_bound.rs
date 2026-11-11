use vstd::prelude::*;

verus! {

spec fn checked_increment(value: u64) -> Option<u64> {
    if value < u64::MAX {
        Some((value + 1) as u64)
    } else {
        None
    }
}

fn increment_checked(value: u64) -> (result: Option<u64>)
    requires
        value < u64::MAX,
    ensures
        result == checked_increment(value),
{
    Some(value + 1)
}

} // verus!
fn main() {}
