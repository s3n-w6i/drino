pub mod api;

use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use std::time::Duration;
use actix_cors::Cors;
use actix_files::Files;
use actix_web::{web, App, HttpServer};
use actix_web_static_files::ResourceFiles;
use tokio::time::interval;
use common::types::config::Config;
use api::v1::{config_api, stats_api, status_api};
use api::v1::status::{Job, JobStatus, StatusBroadcaster};

// Import the statically built dashboard files
include!(concat!(env!("OUT_DIR"), "/generated.rs"));

/// This generates a tls configuration for localhost use
/// Do not use in production
fn tls_cfg() -> rustls::ServerConfig {
    let mut certs_file = BufReader::new(File::open("cert.pem").unwrap());
    let mut key_file = BufReader::new(File::open("key.pem").unwrap());

    let tls_certs = rustls_pemfile::certs(&mut certs_file)
        .collect::<actix_web::Result<Vec<_>, _>>()
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


pub async fn run_server(config: Config) -> std::io::Result<()> {
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:5173")
            .allowed_origin("https://hoppscotch.io");

        let frontend_files = generate();

        let status_broadcaster = StatusBroadcaster::create();

        let broadcaster: Arc<StatusBroadcaster> = Arc::clone(&status_broadcaster);
        actix_web::rt::spawn(async move {
            let mut interval = interval(Duration::from_secs(6));

            broadcaster.update_silently(Job::HarvestData, JobStatus::Running);
            broadcaster.broadcast().await.unwrap();

            interval.tick().await;
            broadcaster.update_silently(Job::HarvestData, JobStatus::Succeeded);
            broadcaster.update_silently(Job::ImportData, JobStatus::Running);
            broadcaster.broadcast().await.unwrap();

            interval.tick().await;
            broadcaster.update_silently(Job::ImportData, JobStatus::Succeeded);
            broadcaster.update_silently(Job::ValidateData, JobStatus::Running);
            broadcaster.broadcast().await.unwrap();

            interval.tick().await;
            broadcaster.update_silently(Job::ValidateData, JobStatus::Succeeded);
            broadcaster.update_silently(Job::PreprocessingClustering, JobStatus::Running);
            broadcaster.broadcast().await.unwrap();

            interval.tick().await;
            interval.tick().await;
            broadcaster.update_silently(Job::PreprocessingClustering, JobStatus::Failed);
            broadcaster.broadcast().await.unwrap();
        });

        App::new()
            .wrap(cors)
            // Make config available in all handlers
            .app_data(web::Data::new(config.clone()))
            // Build a global channel to send status data
            .app_data(web::Data::new(Arc::clone(&status_broadcaster)))
            // API endpoints
            .service(stats_api)
            .service(config_api)
            .service(status_api)
            // Static files
            .service(Files::new("/data-files", "../data").prefer_utf8(true))
            // Serve the frontend. This is a catchall, so it must be defined last.
            .service(ResourceFiles::new("/", frontend_files).resolve_not_found_to_root())
    })
        .bind_rustls_0_23("127.0.0.1:3001", tls_cfg())?
        .run()
        .await
}