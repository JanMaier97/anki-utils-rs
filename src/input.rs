use crate::field_validation::{ValidationConfig, ValidationType};
use crate::{CliArgs, CliValidationType};
use anyhow::{anyhow, Context, Result};
use serde_json;
use std::fs::File;
use std::io::BufReader;

pub fn create_validation_config(cli: &CliArgs) -> Result<ValidationConfig> {
    let f = File::open(&cli.config_file)?;
    let reader = BufReader::new(f);
    let mut config: ValidationConfig = serde_json::from_reader(reader)
        .with_context(|| format!("Failed to read config file from '{}'", cli.config_file))?;

    apply_cli_args_on_config(&mut config, cli)?;

    Ok(config)
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

fn map_validation_type(validation_type: &ValidationType) -> CliValidationType {
    match validation_type {
        ValidationType::Required => CliValidationType::Required,
        ValidationType::ValueList(_) => CliValidationType::ValueList,
        ValidationType::MustNotInclude(_) => CliValidationType::MustNotInclude,
    }
}
