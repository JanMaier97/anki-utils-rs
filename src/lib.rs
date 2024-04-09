use std::error::Error;

pub mod anki;
pub mod field_validation;

pub type MyResult<T> = Result<T, Box<dyn Error>>;
