/// Deliberately violating fixture for the checked Semgrep producer profile.
///
/// The empty expect reason violates the checked `expect-without-rationale` rule.
pub fn parse_byte() -> u8 {
    "1".parse::<u8>().expect("")
}
