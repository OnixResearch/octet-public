let allowed = ["deny", "error"] in
let value = "unknown" in
if std.array.elem value allowed then value else error "unknown-value"
