struct State;

fn is_not_ready(_state: &State) -> bool {
    false
}

fn has_no_capacity() -> bool {
    true
}

fn is_ready(_state: &State) -> bool {
    true
}

fn notify(_state: &State) -> bool {
    true
}

fn not(value: bool) -> bool {
    !value
}

fn locals() {
    let no_capacity = true;
    let no_progress: usize = 3;
    let _ = (no_capacity, no_progress);
}

// Should warn with a review-required rename: `is_ready` is already a parameter,
// so the positive name would capture it.
fn positive_name_is_taken(is_ready: bool) -> bool {
    let is_not_ready = !is_ready;
    is_not_ready
}

// Each local below should warn with a review-required rename: the rewrite
// cannot negate every read of the binding.
fn review_required_locals(values: &[u8]) -> bool {
    // Mutated after its declaration.
    let mut no_values = values.is_empty();
    no_values &= values.len() < 2;
    // Declared without an initializer.
    let no_items;
    no_items = values.len() == 0;
    // Borrowed explicitly.
    let lacks_header = values.first() != Some(&0);
    let header_ref = &lacks_header;
    // Borrowed through method autoref.
    let cannot_parse = values.len() > 4;
    let is_copy = cannot_parse.clone();
    // Read inside a macro.
    let wont_fit = values.len() > 8;
    let fit_text = format!("{wont_fit}");
    no_values && no_items && *header_ref && is_copy && fit_text.is_empty()
}

macro_rules! bail_if {
    ($cond:tt) => {
        if $cond {
            return 0;
        }
    };
}

// Should warn with a review-required rename: a `tt` argument keeps its source
// span, so `!has_items` (two tokens) no longer matches the arm, and
// `(!has_items)` as the macro's `if` condition trips rustc's `unused_parens`,
// whose own fix removes the parentheses again.
fn macro_token_tree_use(len: usize) -> u8 {
    let no_items = len == 0;
    bail_if!(no_items);
    1
}

macro_rules! require {
    ($flag:ident) => {
        if !$flag {
            return;
        }
    };
}

macro_rules! require_path {
    ($flag:path) => {
        if !$flag {
            return;
        }
    };
}

// Should warn with a review-required rename: neither `!has_values` nor
// `(!has_values)` is an identifier or a path. rustc gives `$x:ident` and
// `$x:path` arguments a span from the macro expansion, so the read is not
// rewritten.
fn macro_ident_use(values: &[u8]) {
    let no_values = values.is_empty();
    require!(no_values);
}

fn macro_path_use(values: &[u8]) {
    let no_entries = values.is_empty();
    require_path!(no_entries);
}

// Should NOT warn: async fn parameters are not `let` bindings, even though
// rustc lowers them into `let` statements inside the async body.
async fn async_parameter(no_values: bool) -> bool {
    no_values
}

fn main() {
    let state = State;
    let _ = is_not_ready(&state);
    let _ = has_no_capacity();
    let _ = is_ready(&state);
    let _ = notify(&state);
    let _ = not(true);
    locals();
    let _ = positive_name_is_taken(true);
    let _ = review_required_locals(&[1, 2]);
}
