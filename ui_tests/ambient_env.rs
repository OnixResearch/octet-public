fn read_from_ambient_environment() -> Option<String> {
    std::env::var("OCTET_HOME").ok()
}

fn mutate_ambient_environment() {
    unsafe {
        std::env::set_var("OCTET_MODE", "checked");
    }
}

fn parse_port(value: Option<String>) -> Option<u16> {
    value.and_then(|text| text.parse().ok())
}

fn wrapped_ambient_reads() -> (Option<u16>, Option<Result<String, std::env::VarError>>, String) {
    let port = parse_port(std::env::var("PORT").ok());
    let profile = Some(std::env::var("OCTET_PROFILE"));
    let debug = format!("{:?}", std::env::var_os("OCTET_DEBUG"));
    (port, profile, debug)
}

fn main() {
    let _value = read_from_ambient_environment();
    mutate_ambient_environment();
    let _wrapped = wrapped_ambient_reads();
}
