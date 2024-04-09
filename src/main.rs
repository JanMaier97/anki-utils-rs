use anki_utils::field_validation;
use anki_utils::field_validation::ValidationConfig;

fn main() {
    let config = ValidationConfig::new();
    field_validation::execute(config);
}
