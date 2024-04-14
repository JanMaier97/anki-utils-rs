use anki_utils::{run, CliArgs};
use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = CliArgs::parse();
    if let Err(error) = run(cli) {
        eprintln!("{}", error);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
