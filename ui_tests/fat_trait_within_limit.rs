trait NarrowPort {
    fn a(&self);
    fn b(&self);
}

// Should NOT warn: only the three associated fns count. Doc text, string
// literals, and helper fns nested in default bodies are not trait methods.
trait DocumentedPort {
    /// Callers pass a `fn name()` callback; see `fn render` and `fn flush`.
    fn render(&self) -> String {
        fn header() -> &'static str {
            "fn header"
        }
        fn footer() -> &'static str {
            "fn footer"
        }
        format!("{}{}", header(), footer())
    }

    /// Unlike `fn render`, this keeps `fn ` tokens in comments: fn a, fn b.
    fn flush(&self) -> usize {
        fn pending() -> usize {
            0
        }
        pending()
    }

    fn label(&self) -> &'static str {
        "fn label fn label fn label"
    }
}

fn main() {}
