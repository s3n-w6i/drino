use actix_web::{get, web, Responder, Result};

#[get("/api/v1/config")]
pub(crate) async fn config() -> Result<impl Responder> {
    Ok(web::Json(None))
}
