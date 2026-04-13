mod config;
mod domain;
mod runner;

use std::path::PathBuf;
use std::process::ExitCode;

use config::Config;
use domain::{Formatter as _, FormatterError, extension_of};
use runner::CommandRunner;

fn main() -> ExitCode {
    let (path, check) = match parse_args() {
        Ok(v) => v,
        Err(msg) => {
            eprintln!("usage: fmtx [--check] <file>");
            eprintln!("error: {msg}");
            return ExitCode::FAILURE;
        }
    };

    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("fmtx: {e}");
            return ExitCode::FAILURE;
        }
    };

    let ext = match extension_of(&path) {
        Ok(e) => e,
        Err(FormatterError::NoExtension(_)) => return ExitCode::SUCCESS, // nothing to do
        Err(e) => {
            eprintln!("fmtx: {e}");
            return ExitCode::FAILURE;
        }
    };

    let entry = match config.get(&ext) {
        Some(e) => e,
        None => return ExitCode::SUCCESS, // no formatter configured — not an error
    };

    let runner = CommandRunner { entry };
    match runner.format(&path, check) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("fmtx: {e}");
            ExitCode::FAILURE
        }
    }
}

fn parse_args() -> Result<(PathBuf, bool), String> {
    let mut args = std::env::args().skip(1).peekable();
    let mut check = false;

    if args.peek().map(|a| a == "--check").unwrap_or(false) {
        check = true;
        args.next();
    }

    let file = args
        .next()
        .ok_or_else(|| "no file path provided".to_string())?;

    Ok((PathBuf::from(file), check))
}
