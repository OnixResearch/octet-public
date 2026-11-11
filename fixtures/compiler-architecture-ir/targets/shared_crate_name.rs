pub struct Settings {
    pub verbose: bool,
}

pub fn run(settings: &Settings) -> bool {
    settings.verbose
}

fn main() {
    let _ = run(&Settings { verbose: false });
}
