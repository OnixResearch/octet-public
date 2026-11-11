#![warn(raw_domain_value)]
#![allow(non_trait_imports, reason = "fixture targets another lint; the import and path-naming lints have their own fixtures")]

use std::collections::BTreeMap;

struct SessionId(String);

// Should warn: this exact core field remains raw.
struct RawDomainFixture {
    session: String,
}

// Should warn: this exact core map key remains raw.
struct RawMapFixture {
    sessions: BTreeMap<String, u64>,
}

// Should warn: this exact enum payload remains raw.
enum RawEvent {
    Open { session: String },
}

// Should warn: this exact parameter remains raw.
fn open_session(session: String) -> SessionId {
    SessionId(session)
}

// Should warn: this exact result remains raw.
fn current_session() -> String {
    String::new()
}

// Should not warn: this unregistered wire field stays structural.
struct WireDto {
    session: String,
}

// Should not warn: this core field uses a nominal type.
struct TypedCore {
    session: SessionId,
}

mod hashed {
    use std::collections::HashMap;

    // Should warn: a `HashMap` key is a map key like a `BTreeMap` key.
    pub struct RawMapFixture {
        pub sessions: HashMap<String, u64>,
    }
}

// `borrowed_argument_types` would flag `&String`; this fixture only covers raw domain values.
#[allow(borrowed_argument_types)]
mod borrowed {
    static SESSION: String = String::new();

    // Should warn: a borrowed parameter still exposes raw storage.
    pub fn open_session(session: &String) -> usize {
        session.len()
    }

    // Should warn: a borrowed result still exposes raw storage.
    pub fn current_session() -> &'static String {
        &SESSION
    }
}

// `borrowed_argument_types` recommends `&str` over `&String`; the view still exposes raw storage.
mod viewed {
    // Should warn: a borrowed string view parameter still exposes raw storage.
    pub fn open_session(session: &str) -> usize {
        session.len()
    }

    // Should warn: a borrowed string view result still exposes raw storage.
    pub fn current_session() -> &'static str {
        "session"
    }
}

fn main() {
    let raw = RawDomainFixture { session: current_session() };
    let map = RawMapFixture { sessions: BTreeMap::new() };
    let event = RawEvent::Open { session: raw.session.clone() };
    let wire = WireDto { session: raw.session.clone() };
    let typed = TypedCore { session: open_session(raw.session) };
    let RawEvent::Open { session } = event;
    let _ = (map.sessions, session, wire.session, typed.session.0);
}
