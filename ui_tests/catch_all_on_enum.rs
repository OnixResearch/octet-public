enum Permission {
    Read,
    Write,
    Admin,
}

fn check_permission(p: Permission) -> bool {
    // Should warn: catch-all _ => false on enum
    match p {
        Permission::Read => true,
        _ => false,
    }
}

fn check_with_comment(p: Permission) -> bool {
    // Should NOT warn: catch-all has justification comment
    match p {
        Permission::Read => true,
        // All other permissions are denied in read-only mode
        _ => false,
    }
}

fn check_string(s: &str) -> bool {
    // Should NOT warn: not an enum match
    match s {
        "yes" => true,
        _ => false,
    }
}

fn main() {
    check_permission(Permission::Read);
    check_with_comment(Permission::Read);
    check_string("yes");
}
