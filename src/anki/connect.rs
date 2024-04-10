use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{collections::HashMap, error::Error};

pub type AnkiConnectResult<T> = Result<T, Box<dyn Error>>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Field {
    pub value: String,
    order: u32,
}

type NoteId = u64;
type ModelId = u64;

type Fields = HashMap<String, Field>;

pub struct AnkiConnect {
    version: u32,
    client: reqwest::blocking::Client,
    hostname: String,
}

#[derive(Deserialize)]
struct AnkiConnectResponse<T> {
    result: Option<T>,
    error: Option<String>,
}

#[derive(Serialize)]
struct AnkiconnectRequest<'a, T> {
    version: u32,
    action: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<T>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewNote<'a> {
    deck_name: &'a str,
    model_name: &'a str,
    fields: Fields,
    tags: Vec<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NoteInfo {
    pub note_id: NoteId,
    model_name: String,
    tags: Vec<String>,
    pub fields: Fields,
}

#[derive(Serialize)]
struct QueryRequest<'a> {
    query: &'a str,
}

#[derive(Serialize)]
struct NoteIdsRequest<'a> {
    notes: &'a [NoteId],
}

impl AnkiConnect {
    pub fn new() -> Self {
        Self {
            version: 6,
            client: reqwest::blocking::Client::new(),
            hostname: "http://172.18.48.1".to_string(),
        }
    }

    fn invoke<TParams, TResult>(
        &self,
        action: &str,
        params: Option<TParams>,
    ) -> AnkiConnectResult<TResult>
    where
        TParams: Serialize,
        TResult: DeserializeOwned,
    {
        let request = AnkiconnectRequest {
            version: self.version,
            action,
            params,
        };

        let address = format!("{}:8765", self.hostname);

        let response = self.client.post(&address).json(&request).build()?;

        dbg!(response.body());

        let response = self
            .client
            .post(&address)
            .json(&request)
            .send()?
            .json::<AnkiConnectResponse<TResult>>()?;

        if let Some(error) = response.error {
            return Err(error.into());
        }

        let Some(result) = response.result else {
            return Err("Anki connect returned no result".into());
        };

        Ok(result)
    }

    pub fn find_notes(&self, query: &str) -> AnkiConnectResult<Vec<NoteId>> {
        let notes: Vec<NoteId> = self.invoke("findNotes", Some(QueryRequest { query }))?;
        // let notes: Vec<NoteId> = self.invoke("findNotes", query )?;

        return Ok(notes);
    }

    pub fn model_names_and_ids(&self) -> AnkiConnectResult<HashMap<String, u64>> {
        let models = self.invoke::<(), _>("modelNamesAndIds", None)?;

        return Ok(models);
    }

    pub fn notes_info(&self, note_ids: &[NoteId]) -> AnkiConnectResult<Vec<NoteInfo>> {
        let notes = self.invoke("notesInfo", Some(NoteIdsRequest { notes: note_ids }))?;

        return Ok(notes);
    }
}
