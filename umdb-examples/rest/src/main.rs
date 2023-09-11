use std::process::exit;

use actix_cors::Cors;
use tokio::sync::mpsc;
use actix_web::{App, HttpServer, web};

use umdb::rest;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let (termination_signal_sender, mut termination_signal_receiver) = mpsc::unbounded_channel();

    let umdb_instance = rest::create_umdb_handle(termination_signal_sender.downgrade());
    let port          = 8000;

    umdb_instance.write().unwrap().umdb.configuration.adb_command = Some("adb".to_string());

    env_logger::init();

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

        error = termination_signal_receiver.recv() => {
            if let Some(error) = error {
                eprintln!("A fatal error occurred: {:?}", error);

                exit(1)
            }
        }
    );

    Ok(())
}
