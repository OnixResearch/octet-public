// The positive name must hold the negated value. These const assertions are
// evaluated when the original and the autofixed crate compile, so a rename that
// inverts a binding's value fails the autofix rail.
const fn multiple_uses() -> (bool, bool, bool) {
    let is_not_ready = false;
    (is_not_ready, is_not_ready, !is_not_ready)
}

const fn binary_initializer(len: usize) -> bool {
    let has_no_items = len == 0;
    has_no_items
}

const fn negated_initializer(len: usize) -> bool {
    let is_not_empty = !(len == 0);
    is_not_empty
}

macro_rules! ensure {
    ($cond:tt) => {
        if !$cond {
            return 0;
        }
    };
}

// A `tt` argument keeps its source span; the negated read must stay one token
// tree, so it becomes `(!has_items)`, and `!(!has_items)` keeps its parentheses.
const fn macro_token_tree_use(len: usize) -> u8 {
    let has_no_items = len == 0;
    ensure!(has_no_items);
    1
}

// An `expr` argument: `!has_no_values` collapses to the positive name.
const fn macro_expr_use(values: &[u8]) {
    let has_no_values = values.is_empty();
    assert!(!has_no_values);
}

const _: () = {
    let (a, b, c) = multiple_uses();
    assert!(!a);
    assert!(!b);
    assert!(c);
    assert!(binary_initializer(0));
    assert!(!binary_initializer(3));
    assert!(negated_initializer(3));
    assert!(!negated_initializer(0));
    assert!(macro_token_tree_use(0) == 1);
    assert!(macro_token_tree_use(3) == 0);
    macro_expr_use(&[1]);
};

fn closure_use() -> usize {
    let is_not_empty = true;
    let values = [1, 2, 3];
    values.iter().filter(|_| is_not_empty).count()
}

fn async_block_use() {
    let is_not_done = true;
    let _future = async move { is_not_done };
}

struct Flags {
    is_not_hidden: bool,
}

fn shorthand_field_use() -> bool {
    let is_not_hidden = true;
    let flags = Flags { is_not_hidden };
    flags.is_not_hidden
}

fn method_receiver_use() -> Option<u8> {
    let is_not_ready = false;
    is_not_ready.then_some(1)
}

fn main() {
    let _ = multiple_uses();
    let _ = closure_use();
    async_block_use();
    let _ = shorthand_field_use();
    let _ = method_receiver_use();
}
