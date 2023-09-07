use std::sync::RwLock;

use actix_web::web::Data;

use crate::Umdb;

pub type UmdbHandle = Data<RwLock<Umdb>>;

pub fn create_umdb_handle() -> UmdbHandle {
    Data::new(RwLock::new(Umdb::new()))
}

