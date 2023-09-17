use std::net::IpAddr;

use actix_web::{error::ErrorBadRequest, HttpRequest, Responder, Result, web};

use crate::{adb::{executable::check_adb, device::adb_devices, links::adb_open_deep_link, connect::adb_connect}, core::System};
use super::{ActixUmdbHandle, error_handling::{format_error, make_system_unsupported_reponse, MissingHeaderError, MalformedHeaderError}, headers::read_system_header, read_handle};

pub fn configure(config: &mut web::ServiceConfig, umdb: ActixUmdbHandle) {
    config
    .route("/devices", web::get().to(list_devices))
    .route("/configuration", web::get().to(get_config))
    .route("/device/{id}/link", web::post().to(open_deep_link))
    .route("/executable/check", web::get().to(check_executable))
    .route("/device/{id}/connection", web::post().to(connect_tcpip))
    .app_data(umdb);
}

async fn get_config(actix_handle: ActixUmdbHandle) -> Result<impl Responder> {
    let handle_guard = read_handle(&actix_handle)?;

    Ok(web::Json(handle_guard.umdb.configuration.clone()))
}

// This route is dangerous! This allows the called to run any program on the server.
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

async fn list_devices(request: HttpRequest, actix_handle: ActixUmdbHandle) -> Result<impl Responder> {
    let system = read_system_header(&request).map_err(|error| {
        ErrorBadRequest(format_error(error))
    })?;

    if system != System::Android {
        return Err(make_system_unsupported_reponse());
    }

    let handle_guard = read_handle(&actix_handle)?;

    let devices = adb_devices(&handle_guard.umdb.configuration)
    .await
    .map_err(|error| ErrorBadRequest(format_error(error)))?;

    Ok(web::Json(devices))
}

// This route is dangerous! This allows the called to run any program on the server.
async fn open_deep_link(path: web::Path<String>, request: HttpRequest, actix_handle: ActixUmdbHandle, body: web::Bytes) -> Result<impl Responder> {
    let device_id = path.into_inner();

    let device_id_buffer = body.to_vec();

    let link = String::from_utf8_lossy(&device_id_buffer);

    let system = read_system_header(&request).map_err(|error| {
        ErrorBadRequest(format_error(error))
    })?;

    if system != System::Android {
        return Err(make_system_unsupported_reponse());
    }

    let handle_guard = read_handle(&actix_handle)?;

    let devices = adb_open_deep_link(&handle_guard.umdb.configuration, &device_id, &link).map_err(|error| {
        ErrorBadRequest(format_error(error))
    })?;

    Ok(web::Json(devices))
}

async fn connect_tcpip(path: web::Path<String>, request: HttpRequest, actix_handle: ActixUmdbHandle) -> Result<impl Responder> {
    let device_id = path.into_inner();
    let port_header_name = "port";
    let ip_header_name = "ip";

    let system = read_system_header(&request).map_err(|error| {
        ErrorBadRequest(format_error(error))
    })?;

    if system != System::Android {
        return Err(make_system_unsupported_reponse());
    }

    let handle_guard = read_handle(&actix_handle)?;

    let ip = request
    .headers()
    .get(ip_header_name)
    .ok_or(ErrorBadRequest(format_error(MissingHeaderError(ip_header_name))))?
    .to_str()
    .map(|ip| ip.parse::<IpAddr>())
    .unwrap()
    .map_err(|_| ErrorBadRequest(format_error(MalformedHeaderError(ip_header_name))))?;

    let port = request
    .headers()
    .get(port_header_name)
    .ok_or(ErrorBadRequest(format_error(MissingHeaderError(port_header_name))))?
    .to_str()
    .map(|port| port.parse::<u16>())
    .unwrap()
    .map_err(|_| ErrorBadRequest(format_error(MalformedHeaderError(port_header_name))))?;

    adb_connect(&handle_guard.umdb.configuration, &device_id, &ip, port)
    .await
    .map_err(|error| ErrorBadRequest(format_error(error)))?;

    Ok("")
}
