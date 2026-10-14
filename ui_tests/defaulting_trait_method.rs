#![warn(defaulting_trait_method)]

trait Writer {
    fn flush(&mut self) {
        // default silently changes implementor contract
    }
}

fn main() {}
