let values = ["deny", "deny"] in
if std.array.length values == std.array.length (std.array.dedup values)
then values
else error "duplicate-value"
