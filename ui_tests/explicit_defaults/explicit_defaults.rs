use std::collections::VecDeque;

#[derive(Default)]
struct LocalState {
    retries: u8,
}

fn external_default_method() {
    let _queue = VecDeque::<u8>::default();
}

fn external_default_trait() {
    let _queue: VecDeque<u8> = Default::default();
}

fn external_default_ufcs() {
    let _queue = <VecDeque<u8> as Default>::default();
}

fn sibling_workspace_default() {
    let _cfg = shared_types::Config::default();
}

fn sibling_workspace_struct_update() {
    let _cfg = shared_types::Config {
        retries: 3,
        ..Default::default()
    };
}

fn local_default_is_allowed() {
    let local = LocalState::default();
    let _ = local.retries;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_allows_external_default() {
        let _queue = VecDeque::<u8>::default();
        let _cfg = shared_types::Config::default();
    }
}

fn main() {
    external_default_method();
    external_default_trait();
    external_default_ufcs();
    sibling_workspace_default();
    sibling_workspace_struct_update();
    local_default_is_allowed();
}
