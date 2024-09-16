use actix_cors::Cors;
use actix_files::Files;
use actix_web::{get, web, App, Result, HttpServer, Responder};
use serde::Serialize;

#[derive(Serialize)]
struct Stats {
    num_stops: usize,
    num_clusters: usize,
    num_routes: usize,
    num_trips: usize,
}

#[get("/api/v1/stats")]
async fn stats() -> Result<impl Responder> {
    let stats = Stats {
        num_stops: 42,
        num_clusters: 2,
        num_routes: 420,
        num_trips: 420000,
    };
    
    Ok(web::Json(stats))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000");
        
        App::new()
            .wrap(cors)
            // Static files
            .service(Files::new("/data-files", "../data").prefer_utf8(true))
            // Stats endpoints
            .service(stats)
    })
        .bind(("127.0.0.1", 3001))?
        .run()
        .await
}