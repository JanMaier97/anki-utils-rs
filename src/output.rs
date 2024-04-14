use crate::field_validation::ValidationResult;
use anyhow::Result;
use std::io;
use std::io::Write;

pub fn print_validation_result(result: &ValidationResult) -> Result<()> {
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
