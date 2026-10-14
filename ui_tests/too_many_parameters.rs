fn connect(host: &str, port: u16, timeout_ms: u64, retries: u32, tls: bool, proxy: &str) {
    let _ = (host, port, timeout_ms, retries, tls, proxy);
}

fn connect_ok(host: &str, port: u16, timeout_ms: u64, retries: u32, tls: bool) {
    let _ = (host, port, timeout_ms, retries, tls);
}

struct Client;

impl Client {
    fn method(&self, host: &str, port: u16, timeout_ms: u64, retries: u32, tls: bool, proxy: &str) {
        let _ = (host, port, timeout_ms, retries, tls, proxy);
    }
}

trait Connector {
    fn connect(&self, host: &str, port: u16, timeout_ms: u64, retries: u32, tls: bool, proxy: &str);
}

impl Connector for Client {
    fn connect(&self, host: &str, port: u16, timeout_ms: u64, retries: u32, tls: bool, proxy: &str) {
        let _ = (host, port, timeout_ms, retries, tls, proxy);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_helper(a: &str, b: &str, c: &str, d: &str, e: &str, f: &str) {
        let _ = (a, b, c, d, e, f);
    }
}

fn main() {
    let client = Client;
    connect("localhost", 8080, 1_000, 3, true, "proxy");
    connect_ok("localhost", 8080, 1_000, 3, true);
    client.method("localhost", 8080, 1_000, 3, true, "proxy");
    Connector::connect(&client, "localhost", 8080, 1_000, 3, true, "proxy");
}
