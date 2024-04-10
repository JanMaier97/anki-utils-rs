use anki_utils::{field_validation, MyResult};
use anki_utils::field_validation::ValidationConfig;

use std::fs::File;
use std::io::BufReader;

fn main() -> MyResult<()> {
    let f = File::open("./config/test-validation.json")?;
    let reader = BufReader::new(f);
    let config: ValidationConfig = serde_json::from_reader(reader)?;
    let result = field_validation::execute(&config)?;

    dbg!(result);

    Ok(())
}
