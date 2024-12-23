mod api;

use actix_cors::Cors;
use actix_files::Files;
use actix_web::{App, HttpServer, Result};
use actix_web_static_files::ResourceFiles;
use common::util::logging;
use log::{info, LevelFilter};
use std::fs::File;
use std::io::BufReader;
use crate::api::v1::*;

// Import the static dashboard files
include!(concat!(env!("OUT_DIR"), "/generated.rs"));

/// This generates a tls configuration for localhost use
/// Do not use in production
fn tls_cfg() -> rustls::ServerConfig {
    let mut certs_file = BufReader::new(File::open("cert.pem").unwrap());
    let mut key_file = BufReader::new(File::open("key.pem").unwrap());

    let tls_certs = rustls_pemfile::certs(&mut certs_file)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let tls_key = rustls_pemfile::pkcs8_private_keys(&mut key_file)
        .next()
        .unwrap()
        .unwrap();

    rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(tls_certs, rustls::pki_types::PrivateKeyDer::Pkcs8(tls_key))
        .unwrap()
}

async fn run_server() -> std::io::Result<()> {
    HttpServer::new(|| {
        let cors = Cors::default().allowed_origin("http://localhost:5173");

        let frontend_files = generate();

        App::new()
            .wrap(cors)
            // API endpoints
            .service(stats_api)
            .service(config_api)
            // Static files
            .service(Files::new("/data-files", "../data").prefer_utf8(true))
            // Serve the frontend. This is a catchall, so it must be defined last.
            .service(ResourceFiles::new("/", frontend_files).resolve_not_found_to_root())
    })
    .bind_rustls_0_23("127.0.0.1:3001", tls_cfg())?
    .run()
    .await
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    logging::init(LevelFilter::Info);

    run_server().await?;

    info!("Server shut down");

    Ok(())
}
