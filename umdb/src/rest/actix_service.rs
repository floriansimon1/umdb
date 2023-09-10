use actix_web::{error::ErrorBadRequest, HttpRequest, Responder, Result, web};

use crate::{adb::executable::check_adb, core::System};
use super::{ActixUmdbHandle, error_handling::{format_error, MissingHeaderError, make_system_unsupported_reponse}, headers::read_system_header, read_handle};

pub fn configure(config: &mut web::ServiceConfig, umdb: ActixUmdbHandle) {
    config
    .route("/configuration", web::get().to(get_config))
    .route("/executable/check", web::get().to(check_executable))
    .app_data(umdb);
}

async fn get_config(actix_handle: ActixUmdbHandle) -> Result<impl Responder> {
    let handle_guard = read_handle(&actix_handle)?;

    Ok(web::Json(handle_guard.umdb.configuration.clone()))
}

async fn check_executable(request: HttpRequest) -> Result<impl Responder> {
    let path_header_name = "path";

    let system = read_system_header(&request).map_err(|error| {
        ErrorBadRequest(format_error(error))
    })?;

    if system != System::Android {
        return Err(make_system_unsupported_reponse());
    }

    let path = request
    .headers()
    .get(path_header_name)
    .ok_or(ErrorBadRequest(format_error(MissingHeaderError(path_header_name))))?
    .to_str()
    .unwrap();

    check_adb(path).map_err(|error| {
        ErrorBadRequest(format_error(error))
    })?;

    Ok("")
}
