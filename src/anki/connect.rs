use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;

pub type AnkiConnectResult<T> = Result<T, ClientError>;

struct HttpClient {
    client: reqwest::blocking::Client,
}

impl HttpClient {
    fn post(&self, url: &str, payload: &serde_json::Value) -> Result<String, HttpError> {
        let response = self.client.post(url).json(payload).send()?;

        if !response.status().is_success() {
            return Err(HttpError::StatusCode(response));
        }

        response.text().map_err(Into::into)
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        HttpClient {
            client: reqwest::blocking::Client::new(),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("json serialize error: {0}")]
    SerializeJson(serde_json::Error),

    #[error("json parse error: {0}")]
    ParseJson(serde_json::Error),

    #[error("http error: {0}")]
    Http(#[from] HttpError),

    #[error("Anki error: {0}")]
    ReceivedError(String),

    #[error("Anki returned without a result value")]
    MissingResult,
}

#[derive(thiserror::Error, Debug)]
pub enum HttpError {
    #[error("request: {0}")]
    Client(#[from] reqwest::Error),
    #[error("status code {}", reqwest::blocking::Response::status(.0))]
    StatusCode(reqwest::blocking::Response),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Field {
    pub value: String,
    order: u32,
}

type NoteId = u64;
type Fields = HashMap<String, Field>;

pub struct AnkiConnect {
    version: u32,
    client: HttpClient,
    hostname: String,
}

#[derive(Deserialize)]
struct AnkiConnectResponse<T> {
    result: Option<T>,
    error: Option<String>,
}

#[derive(Serialize)]
struct AnkiConnectRequest<'a, T> {
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

#[derive(Serialize)]
struct GuiBrowseRequest<'a> {
    query: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelFieldNamesRequest<'a> {
    model_name: &'a str,
}

impl Default for AnkiConnect {
    fn default() -> Self {
        Self {
            version: 6,
            client: HttpClient::default(),
            hostname: "http://172.18.48.1".to_string(),
        }
    }
}

impl AnkiConnect {
    fn invoke<TParams, TResult>(
        &self,
        action: &str,
        params: Option<TParams>,
    ) -> AnkiConnectResult<TResult>
    where
        TParams: Serialize,
        TResult: DeserializeOwned,
    {
        let request = AnkiConnectRequest {
            version: self.version,
            action,
            params,
        };

        let address = format!("{}:8765", self.hostname);
        let value = serde_json::to_value(request).map_err(ClientError::SerializeJson)?;
        let response = self.client.post(&address, &value)?;

        let response = serde_json::from_str::<AnkiConnectResponse<TResult>>(&response)
            .map_err(ClientError::ParseJson)?;

        if let Some(error) = response.error {
            return Err(ClientError::ReceivedError(error));
        }

        response.result.ok_or(ClientError::MissingResult)
    }

    pub fn find_notes(&self, query: &str) -> AnkiConnectResult<Vec<NoteId>> {
        self.invoke("findNotes", Some(QueryRequest { query }))
    }

    pub fn model_names_and_ids(&self) -> AnkiConnectResult<HashMap<String, u64>> {
        self.invoke::<(), _>("modelNamesAndIds", None)
    }

    pub fn get_field_names(&self, model_name: &str) -> AnkiConnectResult<Vec<String>> {
        let request = ModelFieldNamesRequest { model_name };
        self.invoke("modelFieldNames", Some(request))
    }

    pub fn notes_info(&self, note_ids: &[NoteId]) -> AnkiConnectResult<Vec<NoteInfo>> {
        self.invoke("notesInfo", Some(NoteIdsRequest { notes: note_ids }))
    }

    pub fn browse_notes(&self, note_ids: &[NoteId]) -> AnkiConnectResult<()> {
        let query = note_ids
            .iter()
            .map(|nid| format!("nid:{}", nid))
            .collect::<Vec<_>>()
            .join(" or ");

        self.invoke::<_, Vec<u64>>("guiBrowse", Some(GuiBrowseRequest { query: &query }))?;

        Ok(())
    }
}
