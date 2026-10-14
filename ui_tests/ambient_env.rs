fn read_from_ambient_environment() -> Option<String> {
    std::env::var("OCTET_HOME").ok()
}

fn mutate_ambient_environment() {
    unsafe {
        std::env::set_var("OCTET_MODE", "checked");
    }
}

fn main() {
    let _value = read_from_ambient_environment();
    mutate_ambient_environment();
}
