#![warn(raw_domain_value)]

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

fn main() {
    let raw = RawDomainFixture { session: current_session() };
    let map = RawMapFixture { sessions: BTreeMap::new() };
    let event = RawEvent::Open { session: raw.session.clone() };
    let wire = WireDto { session: raw.session.clone() };
    let typed = TypedCore { session: open_session(raw.session) };
    let RawEvent::Open { session } = event;
    let _ = (map.sessions, session, wire.session, typed.session.0);
}
