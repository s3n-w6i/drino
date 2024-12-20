use actix_cors::Cors;
use actix_files::Files;
use actix_web::error::ErrorInternalServerError;
use actix_web::{get, web, App, HttpServer, Responder, Result};
use polars::frame::UniqueKeepStrategy;
use polars::prelude::{col, AnyValue, LazyCsvReader, LazyFileListReader, PolarsError};
use serde::Serialize;
use std::fs::File;
use std::io::BufReader;
use actix_web_static_files::ResourceFiles;

// Import the static dashboard files
include!(concat!(env!("OUT_DIR"), "/generated.rs"));

#[derive(Serialize)]
struct Stats {
    num_stops: u32,
    num_clusters: u32,
}

#[get("/api/v1/stats")]
async fn stats() -> Result<impl Responder> {
    fn collect_stats() -> Result<Stats, PolarsError> {
        let clustered_stops = LazyCsvReader::new("../data/tmp/stp/stops_clustered.csv").finish()?;
        
        let counts = clustered_stops.clone().count().collect()?;
        let num_stops = match counts.get_columns().get(0).unwrap().get(0)? {
            AnyValue::UInt32(count) => count,
            _ => { return Err(PolarsError::ComputeError("Failed to calculate num_stops".into())) }
        };
        
        let cluster_count = clustered_stops.clone()
            .select([col("cluster_id")])
            .unique(None, UniqueKeepStrategy::Any)
            .count().collect()?;
        let num_clusters = match cluster_count.get_columns().get(0).unwrap().get(0)? {
            AnyValue::UInt32(count) => count,
            _ => { return Err(PolarsError::ComputeError("Failed to calculate num_clusters".into())) }
        };

        Ok(Stats {
            num_stops,
            num_clusters,
        })
    }
    
    match collect_stats() {
        Ok(stats) => {
            Ok(web::Json(stats))
        },
        Err(err) => {
            match err {
                PolarsError::IO { .. } => {
                    // TODO: Use a better way to determine whether service is ready
                    Err(actix_web::Error::from(ErrorInternalServerError("Unable to read stops_clustered.csv")))
                },
                _ => Err(actix_web::Error::from(ErrorInternalServerError("")))
            }
        }
    }
}

/// This generates a tls configuration for localhost use
/// Do not use in production
fn tls_cfg() -> rustls::ServerConfig {
    let mut certs_file = BufReader::new(File::open("cert.pem").unwrap());
    let mut key_file = BufReader::new(File::open("key.pem").unwrap());

    let tls_certs = rustls_pemfile::certs(&mut certs_file)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let tls_key = rustls_pemfile::pkcs8_private_keys(&mut key_file)
        .next()
        .unwrap()
        .unwrap();

    rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(tls_certs, rustls::pki_types::PrivateKeyDer::Pkcs8(tls_key))
        .unwrap()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let cors = Cors::default()
            .allowed_origin("http://localhost:5173");
        
        let frontend_files = generate();

        App::new()
            .wrap(cors)
            // Static files
            .service(Files::new("/data-files", "../data").prefer_utf8(true))
            // Serve the frontend
            .service(ResourceFiles::new("/", frontend_files))
            // Stats endpoints
            .service(stats)
    })
        .bind_rustls_0_23("127.0.0.1:3001", tls_cfg())?
        .run()
        .await
}