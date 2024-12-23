use actix_web::{get, web, Responder, Result};
use common::types::config::Config;

#[get("/api/v1/config")]
pub(crate) async fn config(config: web::Data<Config>) -> Result<impl Responder> {
    Ok(web::Json(config))
}
