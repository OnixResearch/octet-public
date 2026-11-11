use std::error::Error;

mod execute;
mod io;
mod plan;
mod scan;
mod unpack;
mod verify;

pub(crate) fn run_compact(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let Some(command) = args.first().cloned() else {
        return Err("compact requires archive, verify, or unpack".into());
    };
    let remaining = args.into_iter().skip(1).collect();
    match command.as_str() {
        "archive" => plan::run_compact_archive(remaining),
        "verify" => verify::run_compact_verify(remaining),
        "unpack" => unpack::run_compact_unpack(remaining),
        _ => Err(format!("unknown compact command: {command}").into()),
    }
}

#[cfg(test)]
mod tests;
