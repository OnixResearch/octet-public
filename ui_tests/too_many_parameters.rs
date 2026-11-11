#![allow(non_trait_imports, reason = "fixture targets another lint; the import and path-naming lints have their own fixtures")]

use std::pin::Pin;
use std::sync::Arc;

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

impl Client {
    // Should NOT warn: explicit `self: X` receivers are not parameters.
    fn boxed(self: Box<Self>, host: &str, port: u16, timeout_ms: u64, retries: u32, tls: bool) {
        let _ = (self, host, port, timeout_ms, retries, tls);
    }

    fn shared(self: Arc<Self>, host: &str, port: u16, timeout_ms: u64, retries: u32, tls: bool) {
        let _ = (self, host, port, timeout_ms, retries, tls);
    }

    fn pinned(self: Pin<&mut Self>, host: &str, port: u16, timeout_ms: u64, retries: u32, tls: bool) {
        let _ = (self, host, port, timeout_ms, retries, tls);
    }

    fn explicit_ref(self: &Self, host: &str, port: u16, timeout_ms: u64, retries: u32, tls: bool) {
        let _ = (self, host, port, timeout_ms, retries, tls);
    }

    // Should warn: associated functions without a receiver count every input.
    fn build(host: &str, port: u16, timeout_ms: u64, retries: u32, tls: bool, proxy: &str) -> Self {
        let _ = (host, port, timeout_ms, retries, tls, proxy);
        Self
    }
}

trait SharedConnector {
    // Should NOT warn: explicit receivers in trait declarations are not parameters.
    fn connect_shared(self: Arc<Self>, host: &str, port: u16, timeout_ms: u64, retries: u32, tls: bool);
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
    Box::new(Client).boxed("localhost", 8080, 1_000, 3, true);
    Arc::new(Client).shared("localhost", 8080, 1_000, 3, true);
    let mut pinned = Client;
    Pin::new(&mut pinned).pinned("localhost", 8080, 1_000, 3, true);
    Client.explicit_ref("localhost", 8080, 1_000, 3, true);
    let _ = Client::build("localhost", 8080, 1_000, 3, true, "proxy");
}
