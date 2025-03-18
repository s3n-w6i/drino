pub mod api;

use std::path::PathBuf;
use actix_cors::Cors;
use actix_files::Files;
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use actix_web_static_files::ResourceFiles;
use api::v1::status::{Job, JobStatus, StatusBroadcaster};
use api::v1::{config_api, stats_api, status_api};
use common::types::config::Config;
use std::sync::Arc;
use std::time::Duration;
use actix_web::middleware::Logger;
use tokio::time::interval;

// Import the statically built dashboard files
include!(concat!(env!("OUT_DIR"), "/generated.rs"));

pub async fn build_server(
    config: Config,
    data_path: PathBuf,
    disable_signals: bool
) -> std::io::Result<Server> {
    let mut http_server = HttpServer::new(move || {
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
            .wrap(Logger::default().log_target("visualization"))
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
            .service(Files::new("/data-files", data_path.clone()).prefer_utf8(true))
            // Serve the frontend. This is a catchall, so it must be defined last.
            .service(ResourceFiles::new("/", frontend_files).resolve_not_found_to_root())
    })
        .bind("127.0.0.1:3001")?;
    
    if disable_signals {
        http_server = http_server.disable_signals();
    }
    
    Ok(http_server.run())
}