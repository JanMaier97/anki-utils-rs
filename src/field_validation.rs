use std::collections::HashMap;

use crate::anki::connect::{AnkiConnect, NoteInfo};
use crate::MyResult;

pub struct ValidationConfig {
    model_id: String,
    field_validations: HashMap<String, Vec<ValidationType>>,
}

impl ValidationConfig {
    pub fn new() -> Self {
        let mut fields = HashMap::new();

        let validations = vec![
            ValidationType::Required,
            ValidationType::ValueList(vec!["u-Vt".to_string(), "u-Vi".to_string()])
        ];

        fields.insert("Type".to_string(), validations);

        ValidationConfig {
            model_id: "1576932125743".to_string(),
            field_validations: fields,
        }
    }
}

#[derive(Debug)]
pub struct ValidationResult {
    pub total_note_count: usize,
    pub failed_note_count: usize,
    pub validation_errors: HashMap<u64, HashMap<String, ValidationType>>,
}

pub struct ValidationError {
    message: String,
    note_ids: Vec<u64>,
}

#[derive(Clone, Debug)]
pub enum ValidationType {
    Required,
    ValueList(Vec<String>),
}

impl ValidationType {
    fn is_valid(&self, value: &str) -> bool {
        match self {
            ValidationType::Required => !value.trim().is_empty(),
            ValidationType::ValueList(values) => {
                if value.is_empty() {
                    return true;
                }
                value
                .split(',')
                .inspect(|s| println!("checking if '{}' in list", s))
                .map(|s| s.trim().to_string())
                .all(|s| values.contains(&s))
            }
        }
    }

    fn get_message(&self) -> String {
        match self {
            ValidationType::Required => "Missing required value".to_string(),
            ValidationType::ValueList(values) => format!(
                "Field must only contain valid values: {}",
                values.join(", ")
            ),
        }
    }
}

pub fn execute(config: &ValidationConfig) -> MyResult<ValidationResult> {
    let connector = AnkiConnect::new();

    validate_config(config, &connector)?;

    // let models = connector.model_names_and_ids().unwrap();
    // dbg!(models);

    execute_validation(config, &connector)
}

fn validate_config(config: &ValidationConfig, connector: &AnkiConnect) -> MyResult<()> {
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
        let failed_validations = get_first_failing_validation_per_field(&note, config)?;
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
