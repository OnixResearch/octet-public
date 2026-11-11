#![allow(path_segment_repetition, reason = "fixture targets another lint; the import and path-naming lints have their own fixtures")]

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

    fn assoc_recursion(n: u64) -> u64 {
        // Should warn: `Self::` associated-function recursion.
        if n == 0 { 0 } else { Self::assoc_recursion(n - 1) }
    }

    fn named_assoc_recursion(n: u64) -> u64 {
        // Should warn: `Type::` associated-function recursion.
        if n == 0 { 0 } else { Counter::named_assoc_recursion(n - 1) }
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

trait Walk {
    fn walk(&self, n: u64) -> u64;
}

impl Walk for Counter {
    fn walk(&self, n: u64) -> u64 {
        // Should warn: trait impl method calls itself through the trait.
        if n == 0 { 0 } else { self.walk(n - 1) }
    }
}

impl Walk for u64 {
    fn walk(&self, n: u64) -> u64 {
        // Should warn: UFCS trait call resolves back to this impl.
        if n == 0 { *self } else { Walk::walk(self, n - 1) }
    }
}

struct Wrapper(Counter);

impl Walk for Wrapper {
    fn walk(&self, n: u64) -> u64 {
        // Should NOT warn: delegates to a different impl of the same trait method.
        self.0.walk(n) + Walk::walk(&self.0, n)
    }
}

fn main() {
    factorial(5);
    iterative_factorial(5);
    let counter = Counter;
    counter.method_recursion(5);
    counter.default_helper();
    Counter::assoc_recursion(5);
    Counter::named_assoc_recursion(5);
    counter.walk(5);
    7u64.walk(5);
    Wrapper(Counter).walk(5);
}

struct Tree {
    kids: Vec<Tree>,
}

fn depth(tree: &Tree) -> usize {
    // Should warn: self-call inside a closure body.
    1 + tree.kids.iter().map(|kid| depth(kid)).max().unwrap_or(0)
}

async fn walk_async(n: u64) -> u64 {
    // Should warn: self-call inside an `async fn` body.
    if n == 0 { 0 } else { Box::pin(walk_async(n - 1)).await }
}

fn closure_calls_other(values: &[u64]) -> u64 {
    // Should NOT warn: the closure calls a different function.
    values.iter().map(|value| iterative_factorial(*value)).sum()
}

fn outer_with_nested_item(n: u64) -> u64 {
    // Should NOT warn: the nested item's body is not this function's body,
    // and an anonymous-const body is walked without confusing typeck tables.
    fn nested(n: u64) -> u64 {
        n + 1
    }
    let buffer = [0u8; { core::mem::size_of::<u64>() }];
    nested(n) + buffer.len() as u64
}
