use actix_web::{get, web, Responder, Result};
use common::types::config::Config;

#[get("/api/v1/config")]
pub(crate) async fn config() -> Result<impl Responder> {
    // TODO
    Ok(web::Json(Config::Version1 {
        datasets: vec![],
        dataset_groups: vec![]
    }))
}
