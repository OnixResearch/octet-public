mod octet {
    /// Compatibility API retained for downstream Slotcar callers.
    pub fn octet_config() {}

    // Should NOT warn: the note sits above the attribute, not directly above the item.
    // Compatibility shim for Slotcar callers.
    #[inline]
    pub fn octet_reset() {}

    // Should NOT warn: the note is on the first line of a multi-line doc comment.
    /// Compatibility API retained for downstream Slotcar callers.
    /// New code should call `octet::config`.
    pub fn octet_settings() {}

    // Should warn: a `rapid` note is not a public name note.
    /// Rapid setup for tests.
    pub fn octet_setup() {}
}

fn main() {
    octet::octet_config();
}
