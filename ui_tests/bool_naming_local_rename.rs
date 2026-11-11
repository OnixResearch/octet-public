fn multiple_uses() {
    let active = true;
    let _a = active;
    let _b = active;
    let _c = active;
}

fn closure_use() -> usize {
    let enabled = true;
    let values = [1, 2, 3];
    values.iter().filter(|_| enabled).count()
}

fn async_block_use() {
    let ready = true;
    let _future = async move { ready };
}

struct Flags {
    visible: bool,
}

fn shorthand_field_use() -> bool {
    let visible = true;
    let flags = Flags { visible };
    flags.visible
}

// Should NOT warn: async fn parameters are not `let` bindings, even though
// rustc lowers them into `let` statements inside the async body.
async fn async_parameter(verbose: bool) -> bool {
    verbose
}

fn main() {
    multiple_uses();
    let _ = closure_use();
    async_block_use();
    let _ = shorthand_field_use();
    let _future = async_parameter(true);
}
