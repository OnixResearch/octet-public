fn constructor_unwraps_are_allowed() {
    let _ = Some(1_u32).unwrap();
    let _ = Ok::<u32, &str>(1).unwrap();
}

fn literal_parse_unwrap_is_allowed() {
    let _: std::net::IpAddr = "127.0.0.1".parse().unwrap();
}

fn constructor_expect_still_warns() -> u32 {
    Some(1_u32).expect("constructor expect should still warn")
}

fn invalid_literal_parse_still_warns() -> std::net::IpAddr {
    "not an ip".parse().unwrap()
}

struct LocalParsed;

impl std::str::FromStr for LocalParsed {
    type Err = &'static str;

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        Ok(Self)
    }
}

fn unsupported_target_parse_still_warns() -> LocalParsed {
    "still warns".parse().unwrap()
}

fn non_literal_parse_still_warns(input: &str) -> std::net::IpAddr {
    input.parse().unwrap()
}

fn main() {
    let _ = constructor_expect_still_warns();
    let _ = invalid_literal_parse_still_warns();
    let _ = unsupported_target_parse_still_warns();
    let _ = non_literal_parse_still_warns("127.0.0.1");
}
