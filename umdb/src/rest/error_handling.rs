use std::any;

use actix_web::{error::ErrorBadRequest, Error};
use serde::Serialize;

#[derive(Serialize)]
struct WrappedError<ActualError> {
    #[serde(rename = "type")] error_type: String,
    #[serde(rename = "details")] error: ActualError,
}

#[derive(Serialize)]
pub struct SystemUnsupportedError
{}

#[derive(Serialize)]
pub struct MissingHeaderError<'a>(pub &'a str);

pub fn make_system_unsupported_reponse() -> Error {
    ErrorBadRequest(format_error(SystemUnsupportedError {}))
}

pub fn format_error<T>(error: T) -> serde_json::Value where T: Serialize {
    let error_type = any::type_name::<T>().to_string();

    serde_json::to_value(WrappedError { error, error_type }).unwrap()
}
