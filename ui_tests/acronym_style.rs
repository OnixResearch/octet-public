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

// Should NOT warn: a single capital letter before a capitalized word is a
// one-letter word, not an acronym
struct XAxis {
    data: u32,
}

// Should NOT warn: single-letter word prefix
struct BTreeIndex {
    data: u32,
}

// Should warn: every acronym run is rewritten
struct HTTPRequestJSONBody {
    data: u32,
}

// Should warn: trailing acronyms are acronym runs too
struct ParseJSON {
    data: u32,
}

// Should warn: a two-letter trailing acronym
struct UserID {
    data: u32,
}

// Should warn: an acronym ends before a digit
struct HTTP2Server {
    data: u32,
}

fn main() {
    let _ = RPCHandler { data: 1 };
    let _ = SQLQuery::Select;
    let _ = RpcHandler { data: 1 };
    let _ = XAxis { data: 1 };
    let _ = BTreeIndex { data: 1 };
    let _ = HTTPRequestJSONBody { data: 1 };
}
