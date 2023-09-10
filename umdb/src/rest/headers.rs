use actix_web::HttpRequest;
use serde::Serialize;

use crate::core::System;

#[derive(Debug, Serialize)]
pub enum ReadSystemHeaderError {
    HeaderMissing,
    InvalidValue,
}

pub fn read_system_header(request: &HttpRequest) -> Result<System, ReadSystemHeaderError> {
    let value = request
    .headers()
    .get("system")
    .ok_or(ReadSystemHeaderError::HeaderMissing)?
    .to_str()
    .map_err(|_| ReadSystemHeaderError::InvalidValue)?
    .to_lowercase();

    match value.as_str() {
        "android" => Ok(System::Android),
        "ios"     => Ok(System::Ios),

        _         => Err(ReadSystemHeaderError::InvalidValue),
    }
}
