use anyhow::{anyhow, Result};
use std::collections::HashMap;

use crate::{anki::connect::{AnkiConnect, NoteInfo}, ValidationConfig, ValidationResult, ValidationType};


pub fn execute(config: &ValidationConfig, connector: &AnkiConnect) -> Result<ValidationResult> {
    validate_config(config, connector)?;
    execute_validation(config, connector)
}

fn validate_config(config: &ValidationConfig, connector: &AnkiConnect) -> Result<()> {
    let model_name = connector
        .model_names_and_ids()?
        .into_iter()
        .find(|(_, model_id)| *model_id == config.model_id)
        .map(|(model_name, _)| model_name)
        .ok_or(anyhow!(
            "Failed to find a model with the id {}",
            config.model_id
        ))?;

    let field_names = connector.get_field_names(&model_name)?;
    let invalid_field_names = config
        .field_validations
        .keys()
        .filter(|key| !field_names.contains(key))
        .map(|key| format!("'{}'", key))
        .collect::<Vec<_>>();

    if !invalid_field_names.is_empty() {
        let field_list = invalid_field_names.join(", ");
        return Err(anyhow!(
            "The specified model does not have these fields: {}",
            field_list
        ));
    }

    Ok(())
}

fn execute_validation(
    config: &ValidationConfig,
    connector: &AnkiConnect,
) -> Result<ValidationResult> {
    let query = format!("mid:{}", config.model_id);
    let note_ids = connector.find_notes(&query)?;
    let notes = connector.notes_info(&note_ids)?;

    let mut failed_notes: HashMap<u64, HashMap<String, ValidationType>> = HashMap::new();
    for note in notes.iter() {
        let failed_validations = get_first_failing_validation_per_field(note, config)?;
        if !failed_validations.is_empty() {
            failed_notes.insert(note.note_id, failed_validations);
        }
    }

    let result = ValidationResult {
        total_note_count: notes.len(),
        failed_note_count: failed_notes.len(),
        validation_errors: failed_notes,
    };

    Ok(result)
}

fn get_first_failing_validation_per_field(
    note: &NoteInfo,
    config: &ValidationConfig,
) -> Result<HashMap<String, ValidationType>> {
    let mut result: HashMap<String, ValidationType> = HashMap::new();
    for (field_name, validations) in config.field_validations.iter() {
        for validation in validations {
            let field = note.fields.get(field_name).ok_or(anyhow!(
                "The field '{}' for note type '{}' does not exist",
                field_name,
                config.model_id
            ))?;

            if !validation.is_valid(&field.value) {
                // quickly cloning, should be fixed later
                result.insert(field_name.clone(), validation.clone());
            }
        }
    }

    Ok(result)
}
