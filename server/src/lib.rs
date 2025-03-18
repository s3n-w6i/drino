mod api;

use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use common::types::config::Config;
use log::info;
use routing::raptor::RaptorAlgorithm;
use std::fmt::Display;
use std::sync::Arc;

type ALGORITHM = RaptorAlgorithm;

struct AppData {
    algorithm: ALGORITHM,
    config: Config,
}

pub async fn build<'a>(algorithm: ALGORITHM, config: Config) -> Result<Server, ServerError> {
    info!(target: "server", "Starting API server");

    let app_data = Arc::new(AppData { algorithm, config });

    let server = HttpServer::new(move || {
        let data = Data::new(app_data.clone());
        
        App::new()
            .app_data(data)
            .service(api::v1::routing::endpoint)
    })
    .bind("127.0.0.1:8080")?
    .disable_signals() // We'll handle shutdown ourselves
    .run();

    info!(target: "server", "Started API server");

    Ok(server)
}

#[derive(thiserror::Error, Debug)]
pub enum ServerError {
    Io(#[from] std::io::Error),
}

impl Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
