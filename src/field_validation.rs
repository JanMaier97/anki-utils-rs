use crate::anki::connect::AnkiConnect;
use crate::MyResult;

pub struct ValidationConfig {}

impl ValidationConfig {
    pub fn new() -> Self {
        ValidationConfig {}
    }
}

pub fn execute(config: ValidationConfig) -> MyResult<()> {
    let connector = AnkiConnect::new();
    let query = "mid:1576932125743";
    // let query = "note:Basic";

    let models = connector.model_names_and_ids().unwrap();
    // let note_ids = connector.find_notes(query).unwrap();
    // let notes = connector.notes_info(&note_ids).unwrap();

    dbg!(models);

    // dbg!(note_ids);
    // dbg!(notes);

    Ok(())
}
