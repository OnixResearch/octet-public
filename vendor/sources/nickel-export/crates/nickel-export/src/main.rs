//! `nickel-export` command-line entrypoint.

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if let Err(error) = nickel_export::run(&args) {
        match serde_json::to_string(&error) {
            Ok(rendered) => {
                eprintln!("{rendered}");
            }
            Err(render_error) => {
                eprintln!("nickel-export: {error}; rendering failure: {render_error}");
            }
        }
        std::process::exit(nickel_export::FAILURE_EXIT_CODE);
    }
}
