mod api;

use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use log::info;
use routing::algorithm::RoutingAlgorithm;
use std::fmt::Display;
use std::sync::Arc;

pub async fn build<A: RoutingAlgorithm + Clone + Send + 'static>(
    algorithm: A,
) -> Result<Server, ServerError> {
    info!(target: "server", "Starting API server");

    let server = HttpServer::new(move || {
        let data = Data::new(Arc::new(algorithm.clone()));

        App::new().app_data(Data::clone(&data))
            //.service(api::v1::earliest_arrival::earliest_arrival)
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
