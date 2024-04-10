use anki_utils::{field_validation, MyResult};
use anki_utils::field_validation::ValidationConfig;

fn main() -> MyResult<()> {
    let config = ValidationConfig::new();
    let result = field_validation::execute(&config)?;

    dbg!(result);
    Ok(())
}
