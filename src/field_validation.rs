use serde::Deserialize;
use std::collections::HashMap;

use crate::anki::connect::{AnkiConnect, NoteInfo};
use crate::MyResult;

#[derive(Debug, Deserialize)]
pub struct ValidationConfig {
    model_id: u64,
    field_validations: HashMap<String, Vec<ValidationType>>,
}

#[derive(Debug)]
pub struct ValidationResult {
    pub total_note_count: usize,
    pub failed_note_count: usize,
    pub validation_errors: HashMap<u64, HashMap<String, ValidationType>>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type", content = "check")]
pub enum ValidationType {
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

    pub fn get_message(&self) -> String {
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

pub fn execute(config: &ValidationConfig) -> MyResult<ValidationResult> {
    let connector = AnkiConnect::default();
    validate_config(config, &connector)?;
    execute_validation(config, &connector)
}

fn validate_config(config: &ValidationConfig, connector: &AnkiConnect) -> MyResult<()> {
    let model_name = connector
        .model_names_and_ids()?
        .into_iter()
        .find(|(_, model_id)| *model_id == config.model_id)
        .map(|(model_name, _)| model_name)
        .ok_or(format!(
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
        let msg = format!(
            "The specified model does not have these fields: {}",
            field_list
        );
        return Err(msg.into());
    }

    Ok(())
}

fn execute_validation(
    config: &ValidationConfig,
    connector: &AnkiConnect,
) -> MyResult<ValidationResult> {
    let query = format!("mid:{}", config.model_id);
    let note_ids = connector.find_notes(&query).unwrap();
    let notes = connector.notes_info(&note_ids).unwrap();

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
) -> MyResult<HashMap<String, ValidationType>> {
    let mut result: HashMap<String, ValidationType> = HashMap::new();
    for (field_name, validations) in config.field_validations.iter() {
        for validation in validations {
            let field = note.fields.get(field_name).ok_or(format!(
                "The field '{}' for note type '{}' does not exist",
                field_name, config.model_id
            ))?;
            if !validation.is_valid(&field.value) {
                // quickly cloning, should be fixed later
                result.insert(field_name.clone(), validation.clone());
            }
        }
    }

    Ok(result)
}
