fn will_todo() {
    // Should warn: todo! in production code
    todo!("implement later");
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
