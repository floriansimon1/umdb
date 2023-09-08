use std::process::exit;

use actix_cors::Cors;
use tokio::sync::mpsc;
use actix_web::{App, HttpServer, web};

use umdb::rest;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let (fatal_error_sender, mut fatal_error_receiver) = mpsc::unbounded_channel();

    let umdb_instance = rest::create_umdb_handle(fatal_error_sender.downgrade());
    let port          = 8000;

    println!("Starting server…");

    let server = HttpServer
    ::new(move || {
        let cors = Cors::permissive();

        App::new().wrap(cors).service(
            web
            ::scope("/umdb")
            .configure(|config| rest::actix_service::configure(config, umdb_instance.clone()))
        )
    })
    .bind(("127.0.0.1", port))
    .unwrap_or_else(|_| panic!("Could not bind server to port {port}"));

    println!("Listening for requests on port {port}");

    tokio::select!(
        result = server.run() => { result.expect("An unknown error occurred while running the HTTP server") }

        error = fatal_error_receiver.recv() => {
            if let Some(error) = error {
                eprintln!("A fatal error occurred: {:?}", error);

                exit(1)
            }
        }
    );

    Ok(())
}
