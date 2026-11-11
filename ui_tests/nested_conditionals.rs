fn nested(value: u32) -> bool {
    if value > 0 {
        if value > 1 {
            if value > 2 {
                if value > 3 {
                    return true;
                }
            }
        }
    }

    false
}

fn main() {
    let _ = nested(4);
}

// Should NOT warn: a flat `else if` chain is one level of branching
fn flat_chain(value: u32) -> u32 {
    if value == 0 {
        0
    } else if value == 1 {
        1
    } else if value == 2 {
        2
    } else if value == 3 {
        3
    } else {
        4
    }
}

// Should warn: nesting inside an `else if` arm still counts from the chain level
fn nested_inside_chain(value: u32) -> bool {
    if value == 0 {
        false
    } else if value > 1 {
        if value > 2 {
            if value > 3 {
                if value > 4 {
                    return true;
                }
            }
        }
        false
    } else {
        true
    }
}

struct Router;

impl Router {
    // Should warn: inherent methods are checked like free functions
    fn nested_method(&self, value: u32) -> bool {
        if value > 0 {
            if value > 1 {
                if value > 2 {
                    if value > 3 {
                        return true;
                    }
                }
            }
        }

        false
    }
}

trait NestedDefault {
    // Should NOT warn: required trait methods have no body
    fn required_method(&self, value: u32) -> bool;

    // Should warn: provided trait methods have a body
    fn nested_default_method(&self, value: u32) -> bool {
        if value > 0 {
            if value > 1 {
                if value > 2 {
                    if value > 3 {
                        return true;
                    }
                }
            }
        }

        false
    }
}

// Should warn once, at the local nest below: the conditionals inside expanded
// `assert_eq!` and `debug_assert_eq!` bodies are not nesting the author wrote.
fn assertions_then_deep_nest(value: u32) -> bool {
    if value > 0 {
        if value > 1 {
            assert_eq!(value % 2, 0);
            debug_assert_eq!(value % 3, 0);
        }
    }

    if value > 0 {
        if value > 1 {
            if value > 2 {
                if value > 3 {
                    return true;
                }
            }
        }
    }

    false
}

// Should NOT warn: an assertion at the depth limit adds no level of its own.
fn assertion_at_limit(value: u32) {
    if value > 0 {
        if value > 1 {
            if value > 2 {
                assert_eq!(value, 3);
            }
        }
    }
}

macro_rules! when {
    ($condition:expr, $body:block) => {
        if $condition $body
    };
}

// Should warn: nesting built by a local `macro_rules!` is nesting the author
// wrote, and rustc keeps diagnostics inside local macro bodies.
fn nested_through_local_macro(value: u32) -> bool {
    when!(value > 0, {
        when!(value > 1, {
            when!(value > 2, {
                when!(value > 3, {
                    return true;
                });
            });
        });
    });

    false
}
