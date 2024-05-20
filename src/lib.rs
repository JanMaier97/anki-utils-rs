use crate::input::create_validation_config;
use crate::output::print_validation_result;
use anki::connect::AnkiConnect;
use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};

mod anki;
mod field_validation;
pub mod input;
mod output;

#[derive(Clone, PartialEq, ValueEnum)]
enum CliValidationType {
    Required,
    ValueList,
    MustNotInclude,
}

#[derive(Parser)]
#[command(name = "Anki Field Validator")]
#[command(version, about)]
#[command(
    long_about = "Anki Field Validator is a command line tool to validate notes in anki.\n This tool requires the AnkiConnect plugin to be installed in order to interact with Anki."
)]
pub struct CliArgs {
    config_file: String,
    #[arg(
        long,
        help = "Automatically opens failed notes in the Anki note browser"
    )]
    browse: bool,
    #[arg(
        short,
        long,
        help = "Only validate the fields specified with this argument"
    )]
    fields: Vec<String>,
    #[arg(
        short,
        long,
        value_enum,
        help = "Only apply the validations specified with this argument"
    )]
    validations: Vec<CliValidationType>,
}

pub fn run(cli: CliArgs) -> Result<()> {
    let config = create_validation_config(&cli)?;
    let connector = AnkiConnect::default();
    let result = field_validation::execute(&config, &connector)?;

    print_validation_result(&result)?;

    if cli.browse {
        let note_ids = result.validation_errors.keys().take(997).cloned().collect::<Vec<_>>();
        connector
            .browse_notes(&note_ids)
            .with_context(|| "Failed to open the Anki card browser")?;
    }

    Ok(())
}
