pub struct Closed;
pub struct Open;
pub struct Connection<State> { state: State }

impl Connection<Closed> {
    pub fn open(self) -> Connection<Open> { Connection { state: Open } }
}

impl Connection<Open> {
    pub fn send(self, _bytes: &[u8]) -> Self { self }
}
