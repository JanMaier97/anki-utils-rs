use anki_utils::{field_validation, MyResult};
use anki_utils::field_validation::ValidationConfig;

use clap::Parser;

use std::fs::File;
use std::io::BufReader;

#[derive(Parser)]
#[command(name = "Anki Field Validator")]
#[command(version, about)]
#[command(long_about = "Anki Field Validator is a command line tool to validate notes in anki.\n This tool requires the AnkiConnect plugin to be installed in order to interact with Anki.")]
struct CliArgs {
    config_file: String,
}

fn main() -> MyResult<()> {
    let cli = CliArgs::parse();

    let f = File::open(&cli.config_file)?;
    let reader = BufReader::new(f);
    let config: ValidationConfig = serde_json::from_reader(reader)?;

    let result = field_validation::execute(&config)?;

    dbg!(result);

    Ok(())
}
