use std::process::ExitCode;

use anki_utils::{run, CliArgs};
use clap::Parser;

fn main() -> ExitCode {
    let cli = CliArgs::parse();
    if let Err(error) = run(cli) {
        eprintln!("{}", error);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
