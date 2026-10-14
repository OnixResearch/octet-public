struct UnitId(String);
struct SourceId(String);

fn consume_unit(unit: UnitId) {
    let _ = unit.0;
}

fn main() {
    let source = SourceId(String::from("source"));
    consume_unit(source);
}
