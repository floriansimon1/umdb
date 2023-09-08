use std::sync::{RwLock, RwLockReadGuard};

use tokio::sync::mpsc::WeakUnboundedSender;
use actix_web::{web::Data, error::ErrorInternalServerError};

use crate::Umdb;

#[derive(Debug)]
pub enum FatalError {
    CentralLockPoisoned,
}

pub struct UmdbHandle {
    pub umdb: Umdb,

    fatal_error_sender: WeakUnboundedSender<FatalError>,
}

impl UmdbHandle {
    pub fn signal_fatal(&self, error: FatalError) {
        self
        .fatal_error_sender
        .upgrade()
        .map(|sender| sender.send(error));
    }
}

pub type ActixUmdbHandle = Data<RwLock<UmdbHandle>>;

pub fn create_umdb_handle(fatal_error_sender: WeakUnboundedSender<FatalError>) -> ActixUmdbHandle {
    let umdb = Umdb::new();

    let handle = UmdbHandle {
        umdb,
        fatal_error_sender,
    };

    Data::new(RwLock::new(handle))
}

pub fn read_handle<'a>(actix_handle: &'a ActixUmdbHandle) -> actix_web::Result<RwLockReadGuard<'a, UmdbHandle>> {
    actix_handle
    .read()
    .map_err(|error| {
        let handle = &error.get_ref();

        handle.signal_fatal(FatalError::CentralLockPoisoned);

        ErrorInternalServerError("")
    })
}
