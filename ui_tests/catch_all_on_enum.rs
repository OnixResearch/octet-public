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

impl Permission {
    fn is_write(&self) -> bool {
        // Should warn: `&self` scrutinee has type `&Permission`
        match self {
            Permission::Write => true,
            _ => false,
        }
    }
}

fn check_permission_ref(p: &Permission) -> bool {
    // Should warn: reference scrutinee to an enum
    match p {
        Permission::Admin => true,
        _ => false,
    }
}

fn main() {
    check_permission(Permission::Read);
    check_with_comment(Permission::Read);
    check_string("yes");
    Permission::Read.is_write();
    check_permission_ref(&Permission::Read);
}

fn result_value(res: Result<u64, String>) -> u64 {
    // Should NOT warn: `Result` cannot gain variants
    match res {
        Ok(v) => v,
        _ => 0,
    }
}

fn option_flag(value: Option<bool>) -> bool {
    // Should NOT warn: `Option` cannot gain variants
    match value {
        Some(flag) => flag,
        _ => false,
    }
}

enum Access {
    Grant(u8),
    Revoke { reason: u8 },
    Audit,
    Expire,
    Renew,
}

fn check_access(access: &Access, strict: bool) -> bool {
    // Should warn: the suggestion lists only variants earlier arms leave uncovered
    match access {
        Access::Grant(_) | Access::Audit => true,
        Access::Revoke { reason: 0 } => true,
        Access::Expire if strict => true,
        _ => false,
    }
}
