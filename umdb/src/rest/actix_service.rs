use actix_web::{Result, Responder, web};

use super::{read_handle, ActixUmdbHandle};

pub fn configure(config: &mut web::ServiceConfig, umdb: ActixUmdbHandle) {
    config.service(
        web
        ::resource("/configuration")
        .app_data(umdb)
        .route(web::get().to(get_config))
    );
}

async fn get_config(actix_handle: ActixUmdbHandle) -> Result<impl Responder> {
    let handle_guard = read_handle(&actix_handle)?;

    Ok(web::Json(handle_guard.umdb.configuration.clone()))
}
