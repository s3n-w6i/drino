mod api;

use axum::routing::get;
use axum::Router;
use common::types::config::Config;
use routing::raptor::RaptorAlgorithm;
use std::fmt::Display;
use std::sync::Arc;
use tokio::net::TcpListener;

type ALGORITHM = RaptorAlgorithm;

struct AppData {
    algorithm: ALGORITHM,
    config: Config,
}

pub async fn build<'a>(
    algorithm: ALGORITHM,
    config: Config,
) -> Result<(TcpListener, Router), ServerError> {
    let app_data = Arc::new(AppData { algorithm, config });

    let app = Router::new()
        .route("/api/v1/routing", get(api::v1::routing::endpoint))
        .with_state(app_data);

    let listener = TcpListener::bind("0.0.0.0:8080").await?;

    Ok((listener, app))
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
