fn factorial(n: u64) -> u64 {
    // Should warn: direct recursion
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}

fn iterative_factorial(n: u64) -> u64 {
    // Should NOT warn: uses iteration
    let mut result = 1;
    for i in 2..=n {
        result *= i;
    }
    result
}

struct Counter;

impl Counter {
    fn method_recursion(&self, n: u64) -> u64 {
        // Should warn: direct method recursion.
        if n == 0 { 0 } else { self.method_recursion(n - 1) }
    }

    fn helper(&self) -> u64 {
        1
    }
}

trait Delegates {
    fn required(&self) -> u64;

    fn default_helper(&self) -> u64 {
        // Should NOT warn: different required trait method.
        self.required()
    }
}

impl Delegates for Counter {
    fn required(&self) -> u64 {
        self.helper()
    }
}

fn main() {
    factorial(5);
    iterative_factorial(5);
    let counter = Counter;
    counter.method_recursion(5);
    counter.default_helper();
}
