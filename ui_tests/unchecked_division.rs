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

fn zero_branch_division(total: u64, count: u64) -> u64 {
    // Should warn: the then-branch of `count == 0` divides by zero
    if count == 0 {
        return total / count;
    }
    total
}

fn nonzero_guard_else_branch(total: u64, count: u64) -> u64 {
    // Should warn: the else-branch of `count > 0` has `count == 0`
    if count > 0 {
        0
    } else {
        total / count
    }
}

fn guard_on_other_name(total: u64, len: u64, n: u64) -> u64 {
    // Should warn: the guard checks `len`, not `n`
    if len > 0 {
        total / n
    } else {
        0
    }
}

fn assert_on_other_divisor(a: u64, b: u64, c: u64) -> u64 {
    // Should warn: the assert checks `c`, not `b`
    assert!(c > 0);
    a / b
}

fn later_assert(total: u64, count: u64) -> u64 {
    // Should warn: a later assert cannot justify an earlier division
    let quotient = total / count;
    assert!(count > 0);
    quotient
}

fn zero_check_else_branch(total: u64, count: u64) -> u64 {
    // Should NOT warn: the else-branch of `count == 0` has `count != 0`
    if count == 0 {
        0
    } else {
        total / count
    }
}

fn early_return_zero_check(total: u64, count: u64) -> u64 {
    // Should NOT warn: earlier same-block early return proves `count != 0`
    if count == 0 {
        return 0;
    }
    total / count
}

fn mirrored_guard(total: u64, count: u64) -> u64 {
    // Should NOT warn: mirrored `0 != count` guard
    if 0 != count {
        total / count
    } else {
        0
    }
}

fn at_least_one_guard(total: u64, count: u64) -> u64 {
    // Should NOT warn: `count >= 1` proves nonzero
    if count >= 1 {
        total % count
    } else {
        0
    }
}

fn asserted_divisor(total: u64, count: u64) -> u64 {
    // Should NOT warn: earlier assert proves `count > 0`
    assert!(count > 0);
    total % count
}

fn asserted_divisor_in_loop(values: &[u64], count: u64) -> u64 {
    // Should NOT warn: earlier assert in an enclosing block proves `count != 0`
    assert!(count != 0);
    let mut sum = 0;
    for value in values {
        sum += value / count;
    }
    sum
}

fn main() {
    bare_division(100, 10);
    guarded_division(100, 10);
    literal_division(100);
    zero_branch_division(100, 10);
    nonzero_guard_else_branch(100, 10);
    guard_on_other_name(100, 10, 10);
    assert_on_other_divisor(100, 10, 10);
    later_assert(100, 10);
    zero_check_else_branch(100, 10);
    early_return_zero_check(100, 10);
    mirrored_guard(100, 10);
    at_least_one_guard(100, 10);
    asserted_divisor(100, 10);
    asserted_divisor_in_loop(&[100], 10);
}

fn div_assign_unchecked(total: u64, count: u64) -> u64 {
    // Should warn: `/=` divides by an unchecked variable
    let mut quotient = total;
    quotient /= count;
    quotient
}

fn rem_assign_unchecked(total: u64, count: u64) -> u64 {
    // Should warn: `%=` divides by an unchecked variable
    let mut remainder = total;
    remainder %= count;
    remainder
}

fn div_assign_guarded(total: u64, count: u64) -> u64 {
    // Should NOT warn: earlier early return proves `count != 0`
    if count == 0 {
        return 0;
    }
    let mut quotient = total;
    quotient /= count;
    quotient
}

fn reference_divisor(total: u64, count: &u64) -> u64 {
    // Should warn: `&u64` divisor without zero check
    total / count
}

fn reference_divisor_guarded(total: u64, count: &u64) -> u64 {
    // Should NOT warn: `*count != 0` guards the referenced divisor
    if *count != 0 { total / count } else { 0 }
}

fn dereferenced_divisor_guarded(total: u64, count: &u64) -> u64 {
    // Should NOT warn: `*count > 0` guards `*count`
    if *count > 0 { total % *count } else { 0 }
}

fn float_div_assign(total: f64, count: f64) -> f64 {
    // Should NOT warn: float division does not trap on zero
    let mut quotient = total;
    quotient /= count;
    quotient
}

fn match_zero_arm_binding(total: u64, n: u64) -> Option<u64> {
    // Should NOT warn: the earlier `0` arm leaves only nonzero values for `d`
    match n {
        0 => None,
        d => Some(total / d),
    }
}

fn match_zero_arm_scrutinee(total: u64, n: u64) -> u64 {
    // Should NOT warn: the earlier `0` arm proves the scrutinee nonzero
    match n {
        0 | 1 => 0,
        _ => total % n,
    }
}

fn match_guard_arm(total: u64, n: u64) -> u64 {
    // Should NOT warn: the arm guard proves `d != 0`
    match n {
        d if d != 0 => total / d,
        _ => 0,
    }
}

fn match_nonzero_literal_arm(total: u64, n: u64) -> u64 {
    // Should warn: the earlier arm excludes `1`, not `0`
    match n {
        1 => total,
        d => total / d,
    }
}

fn match_guarded_zero_arm(total: u64, n: u64, strict: bool) -> u64 {
    // Should warn: the `0` arm only runs when `strict` holds
    match n {
        0 if strict => 0,
        d => total / d,
    }
}

fn match_guard_on_other_value(total: u64, n: u64, m: u64) -> u64 {
    // Should warn: the guard checks `m`, not the divisor
    match n {
        d if m != 0 => total / d,
        _ => 0,
    }
}

fn match_division_in_guard(total: u64, n: u64) -> u64 {
    // Should warn: the division runs in the guard, before any check
    match n {
        d if total / d > 1 => 1,
        _ => 0,
    }
}

fn match_zero_arm_other_divisor(total: u64, n: u64, m: u64) -> u64 {
    // Should warn: the `0` arm checks the scrutinee `n`, not the divisor `m`
    match n {
        0 => 0,
        _ => total / m,
    }
}
