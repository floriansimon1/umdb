use actix_web::{error, Result, Responder, web};

use super::UmdbHandle;

pub fn configure(config: &mut web::ServiceConfig, umdb: UmdbHandle) {
    config.service(
        web
        ::resource("/config")
        .app_data(umdb)
        .route(web::get().to(get_config))    
    );
}

async fn get_config(handle: UmdbHandle) -> Result<impl Responder> {
    handle
    .read()
    .map(|umdb| Ok(web::Json(umdb.configuration.clone())))
    .map_err(|_| error::ErrorInternalServerError(""))?
}
