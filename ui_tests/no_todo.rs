#![allow(path_segment_repetition, reason = "fixture targets another lint; the import and path-naming lints have their own fixtures")]

fn will_todo() {
    // Should warn: todo! in production code
    todo!("implement later");
}

fn will_zero_arg_todo() {
    // Should warn: zero-argument todo! expands to a plain call, not a block
    todo!();
}

fn will_path_qualified_todo() {
    // Should warn: path-qualified todo!
    std::todo!("implement later");
}

fn main() {
    // Don't actually call the unfinished function
}

#[test]
fn test_only_todo_is_allowed() {
    todo!("test-only todo");
}

#[cfg(test)]
mod cfg_test_only {
    pub(super) fn todo_in_cfg_test_module_is_allowed() {
        todo!("cfg-test-only todo");
    }
}

struct CfgTestMethodHost;

impl CfgTestMethodHost {
    #[cfg(test)]
    fn todo_in_cfg_test_impl_method_is_allowed() {
        todo!("cfg-test-only method todo");
    }
}

trait CfgTestTraitHost {
    #[cfg(test)]
    fn todo_in_cfg_test_trait_method_is_allowed() {
        todo!("cfg-test-only method todo");
    }
}
