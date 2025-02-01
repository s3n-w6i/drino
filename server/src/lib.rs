use std::fmt::Display;
use routing::algorithm::RoutingAlgorithm;

pub fn serve<A: RoutingAlgorithm>(
    algorithm: A,
) -> Result<(), ServerError> {
    let rt = actix_web::rt::Runtime::new()?;

    let server_handle = rt.spawn(async move {
        /*HttpServer::new(move || {})
            .bind("127.0.0.1:8080");*/
    });

    rt.block_on(server_handle).unwrap();

    todo!()
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