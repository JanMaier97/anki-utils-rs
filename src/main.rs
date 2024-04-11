use anki_utils::{field_validation, MyResult};
use anki_utils::field_validation::ValidationConfig;
use anki_utils::field_validation::ValidationResult;

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

    print_validation_result(&result);

    Ok(())
}

fn print_validation_result(result: &ValidationResult) {
    let field_name_header = "Field Name";
    let error_message_header = "Error Message";

    let max_field_length = result.validation_errors.iter()
           .flat_map(|(_, field_map)| field_map.keys())
           .map(|s| s.len())
           .max()
           .unwrap_or(0)
           .max(field_name_header.len());

    let max_message_length = result.validation_errors.iter()
           .flat_map(|(_, field_map)| field_map.values())
           .map(|e| e.get_message().len())
           .max()
           .unwrap_or(0)
           .max(error_message_header.len());

    println!("|-{}-|-{}-|-{}-|", "-".repeat(20), "-".repeat(max_field_length), "-".repeat(max_message_length));
    println!("| {:<20} | {:<max_field_length$} | {:<max_message_length$} |", "Note Id", field_name_header, error_message_header);
    println!("|-{}-|-{}-|-{}-|", "-".repeat(20), "-".repeat(max_field_length), "-".repeat(max_message_length));

    for (note_id, field_error_map) in result.validation_errors.iter() {
        for  (field_name, error) in field_error_map {
            println!("| {:>20} | {:<max_field_length$} | {:<max_message_length$} |", note_id, field_name, error.get_message());
        }
    }

    println!("|-{}-|-{}-|-{}-|", "-".repeat(20), "-".repeat(max_field_length), "-".repeat(max_message_length));
    println!();
    println!("{} notes have been validated, {} notes failed", result.total_note_count, result.failed_note_count);
}
