// Should warn: acronym-style capitalization
struct RPCHandler {
    data: u32,
}

// Should warn: another acronym
enum SQLQuery {
    Select,
    Insert,
}

// Should NOT warn: proper capitalization
struct RpcHandler {
    data: u32,
}

// Should NOT warn: all-caps is fine
const MAX_RPC: u32 = 100;

fn main() {
    let _ = RPCHandler { data: 1 };
    let _ = SQLQuery::Select;
    let _ = RpcHandler { data: 1 };
}
