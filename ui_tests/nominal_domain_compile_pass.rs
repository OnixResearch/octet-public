use std::marker::PhantomData;

const FIXTURE_TIMEOUT_MS: u64 = 10;

#[derive(Debug)]
struct UnitId(String);

#[derive(Debug)]
struct SourceId(String);

#[derive(Debug)]
struct Milliseconds(u64);

struct UnitDigestRole;
struct Digest<Role> {
    value: String,
    role: PhantomData<Role>,
}

type UnitDigest = Digest<UnitDigestRole>;

struct WireRecord {
    unit: String,
}

impl TryFrom<WireRecord> for UnitId {
    type Error = &'static str;

    fn try_from(wire: WireRecord) -> Result<Self, Self::Error> {
        if wire.unit.is_empty() {
            return Err("unit must not be empty");
        }
        Ok(Self(wire.unit))
    }
}

impl UnitId {
    fn as_str(&self) -> &str {
        &self.0
    }
}

fn consume_unit(unit: UnitId, timeout: Milliseconds) -> (String, u64) {
    (unit.as_str().to_owned(), timeout.0)
}

fn main() {
    let Ok(unit) = UnitId::try_from(WireRecord { unit: String::from("unit") }) else {
        return;
    };
    let source = SourceId(String::from("source"));
    let digest = UnitDigest {
        value: String::from("digest"),
        role: PhantomData,
    };
    let result = consume_unit(unit, Milliseconds(FIXTURE_TIMEOUT_MS));
    let _ = (source.0, digest.value, digest.role, result);
}
