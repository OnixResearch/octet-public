// Should NOT warn: IO is allowlisted in this fixture-local dylint.toml
struct IOError {
    code: u32,
}

// Should warn: RPC is not allowlisted
struct RPCHandler {
    data: u32,
}

fn main() {
    let _ = IOError { code: 1 };
    let _ = RPCHandler { data: 1 };
}
