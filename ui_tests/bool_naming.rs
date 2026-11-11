fn bad_names() {
    // Should warn: bool without predicate prefix
    let active = true;
    let _ = active;

    let ready = false;
    let _ = ready;
}

fn good_names() {
    // Should NOT warn: has predicate prefix
    let is_active = true;
    let _ = is_active;

    let has_data = false;
    let _ = has_data;

    let should_retry = true;
    let _ = should_retry;
}

fn non_bool() {
    // Should NOT warn: not a bool type
    let enabled: u32 = 1;
    let _ = enabled;
}

struct Cache;

impl Cache {
    fn is_dirty(&self) -> bool {
        true
    }
}

// Should warn with a review-required rename: `is_dirty` would shadow the parameter.
fn rename_would_shadow_parameter(is_dirty: bool, cache: &Cache) -> bool {
    let dirty = cache.is_dirty();
    dirty && is_dirty
}

// Should warn with a review-required rename: `is_open` is already a local.
fn rename_would_capture_local() -> bool {
    let open = true;
    let is_open = false;
    open || is_open
}

// Should warn with a review-required rename: `is_set` is an outer parameter
// visible inside the closure.
fn rename_would_shadow_outer_parameter(is_set: bool) -> bool {
    let check = |value: bool| {
        let set = value;
        set && is_set
    };
    check(true)
}

#[allow(non_upper_case_globals, reason = "fixture: a lowercase const occupies the suggested name")]
const is_visible: bool = false;

// Should warn with a review-required rename: `is_visible` names a const, so
// `let is_visible` would become a refutable pattern.
fn rename_would_collide_with_const() -> bool {
    let visible = true;
    visible
}

fn main() {
    bad_names();
    good_names();
    non_bool();
    let _ = rename_would_shadow_parameter(false, &Cache);
    let _ = rename_would_capture_local();
    let _ = rename_would_shadow_outer_parameter(true);
    let _ = rename_would_collide_with_const();
}
