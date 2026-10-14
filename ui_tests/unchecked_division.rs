fn bare_division(total: u64, count: u64) -> u64 {
    // Should warn: division by variable without zero check
    total / count
}

fn guarded_division(total: u64, count: u64) -> u64 {
    // Should NOT warn: zero guard in enclosing if
    if count > 0 {
        total / count
    } else {
        0
    }
}

fn literal_division(total: u64) -> u64 {
    // Should NOT warn: division by literal
    total / 1000
}

fn main() {
    bare_division(100, 10);
    guarded_division(100, 10);
    literal_division(100);
}
