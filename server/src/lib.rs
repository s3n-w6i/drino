use actix_web::rt::task::JoinHandle;
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use routing::algorithm::RoutingAlgorithm;
use std::fmt::Display;
use std::sync::Arc;
use log::info;

pub fn serve<A: RoutingAlgorithm + Clone + Send + 'static>(
    algorithm: A,
) -> Result<(), ServerError> {
    info!(target: "server", "Starting API server");
    
    let rt = actix_web::rt::Runtime::new()?;

    let server_handle: JoinHandle<Result<(), ServerError>> = rt.spawn(async move {
        HttpServer::new(move || {
            let data = Data::new(Arc::new(algorithm.clone()));

            App::new()
                .app_data(Data::clone(&data))
        })
            .bind("127.0.0.1:8080")?
            .run().await?;

        Ok(())
    });

    rt.block_on(server_handle).unwrap()?;

    info!(target: "server", "Shut down API server");
    
    Ok(())
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