use std::{collections::HashMap, fs::File, io::{self, BufReader, Write}};

use anyhow::{anyhow, Context, Result};

use anki::connect::AnkiConnect;
use clap::{Parser, ValueEnum};
use serde::Deserialize;

mod anki;
mod field_validation;

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

#[derive(Debug, Deserialize)]
struct ValidationConfig {
    model_id: u64,
    field_validations: HashMap<String, Vec<ValidationType>>,
}

#[derive(Debug)]
struct ValidationResult {
    total_note_count: usize,
    failed_note_count: usize,
    validation_errors: HashMap<u64, HashMap<String, ValidationType>>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type", content = "check")]
enum ValidationType {
    Required,
    ValueList(Vec<String>),
    MustNotInclude(String),
}

impl ValidationType {
    fn is_valid(&self, value: &str) -> bool {
        match self {
            ValidationType::Required => !value.trim().is_empty(),
            ValidationType::MustNotInclude(invalid_value) => !value.contains(invalid_value),
            ValidationType::ValueList(values) => {
                if value.is_empty() {
                    return true;
                }
                value
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .all(|s| values.contains(&s))
            }
        }
    }

    fn get_message(&self) -> String {
        match self {
            ValidationType::Required => "Missing required value".to_string(),
            ValidationType::MustNotInclude(invalid_value) => {
                format!("Field contains invalid value '{}'", invalid_value)
            }
            ValidationType::ValueList(values) => format!(
                "Field must only contain valid values: {}",
                values.join(", ")
            ),
        }
    }
}


pub fn run(cli: CliArgs) -> Result<()> {
    let f = File::open(&cli.config_file)?;
    let reader = BufReader::new(f);
    let mut config: ValidationConfig = serde_json::from_reader(reader)
        .with_context(|| format!("Failed to read config file from '{}'", cli.config_file))?;

    apply_cli_args_on_config(&mut config, &cli)?;

    let connector = AnkiConnect::default();
    let result = field_validation::execute(&config, &connector)?;

    print_validation_result(&result)?;

    if cli.browse {
        let note_ids = result.validation_errors.keys().cloned().collect::<Vec<_>>();
        connector
            .browse_notes(&note_ids)
            .with_context(|| "Failed to open the Anki card browser")?;
    }

    Ok(())
}

fn apply_cli_args_on_config(config: &mut ValidationConfig, args: &CliArgs) -> Result<()> {
    apply_field_filter(config, args)?;
    apply_validation_filter(config, args);
    Ok(())
}

fn apply_validation_filter(config: &mut ValidationConfig, args: &CliArgs) {
    if args.validations.is_empty() {
        return;
    }

    for (_, validations) in config.field_validations.iter_mut() {
        *validations = validations
            .iter()
            .filter(|t| args.validations.contains(&map_validation_type(t)))
            .cloned()
            .collect::<Vec<_>>();
    }
}

fn map_validation_type(validation_type: &ValidationType) -> CliValidationType {
    match validation_type {
        ValidationType::Required => CliValidationType::Required,
        ValidationType::ValueList(_) => CliValidationType::ValueList,
        ValidationType::MustNotInclude(_) => CliValidationType::MustNotInclude,
    }
}

fn apply_field_filter(config: &mut ValidationConfig, args: &CliArgs) -> Result<()> {
    if args.fields.is_empty() {
        return Ok(());
    }

    let config_fields: Vec<_> = config.field_validations.keys().cloned().collect();

    let invalid_fields: Vec<_> = args
        .fields
        .iter()
        .filter(|f| !config_fields.contains(f))
        .map(|f| format!("'{}'", f))
        .collect();

    if !invalid_fields.is_empty() {
        let field_list = invalid_fields.join(", ");
        return Err(anyhow!(
            "The fields filter must specify fields from the config: {}",
            field_list
        ));
    }

    let fields_to_remove: Vec<_> = config_fields
        .into_iter()
        .filter(|f| !args.fields.contains(f))
        .collect();

    for field in fields_to_remove {
        config.field_validations.remove(&field);
    }

    Ok(())
}

fn print_validation_result(result: &ValidationResult) -> Result<()> {
    let field_name_header = "Field Name";
    let error_message_header = "Error Message";

    let max_field_length = result
        .validation_errors
        .iter()
        .flat_map(|(_, field_map)| field_map.keys())
        .map(|s| s.len())
        .max()
        .unwrap_or(0)
        .max(field_name_header.len());

    let max_message_length = result
        .validation_errors
        .iter()
        .flat_map(|(_, field_map)| field_map.values())
        .map(|e| e.get_message().len())
        .max()
        .unwrap_or(0)
        .max(error_message_header.len());

    let stdout = io::stdout();
    let mut handle = io::BufWriter::new(stdout);

    writeln!(
        handle,
        "|-{}-|-{}-|-{}-|",
        "-".repeat(20),
        "-".repeat(max_field_length),
        "-".repeat(max_message_length)
    )?;
    writeln!(
        handle,
        "| {:<20} | {:<max_field_length$} | {:<max_message_length$} |",
        "Note Id", field_name_header, error_message_header
    )?;
    writeln!(
        handle,
        "|-{}-|-{}-|-{}-|",
        "-".repeat(20),
        "-".repeat(max_field_length),
        "-".repeat(max_message_length)
    )?;

    for (note_id, field_error_map) in result.validation_errors.iter() {
        for (field_name, error) in field_error_map {
            writeln!(
                handle,
                "| {:>20} | {:<max_field_length$} | {:<max_message_length$} |",
                note_id,
                field_name,
                error.get_message()
            )?;
        }
    }

    writeln!(
        handle,
        "|-{}-|-{}-|-{}-|",
        "-".repeat(20),
        "-".repeat(max_field_length),
        "-".repeat(max_message_length)
    )?;
    writeln!(handle)?;
    writeln!(
        handle,
        "{} notes have been validated, {} notes failed",
        result.total_note_count, result.failed_note_count
    )?;

    Ok(())
}
