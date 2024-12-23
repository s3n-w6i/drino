use actix_web::error::ErrorInternalServerError;
use actix_web::{get, web, Responder, Result};
use common::util::df;
use polars::error::PolarsError;
use polars::frame::UniqueKeepStrategy;
use polars::prelude::{col, LazyCsvReader, LazyFileListReader, LazyFrame};
use serde::Serialize;

#[derive(Serialize)]
struct Stats {
    num_stops: u32,
    num_clusters: u32,
    num_trips: u32,
}

#[get("/api/v1/stats")]
pub(crate) async fn stats() -> Result<impl Responder> {
    fn collect_stats() -> Result<Stats, PolarsError> {
        let clustered_stops = LazyCsvReader::new("../data/tmp/stp/stops_clustered.csv").finish()?;
        let num_stops = df::count(clustered_stops.clone())?;

        let clusters = clustered_stops
            .select([col("cluster_id")])
            .unique(None, UniqueKeepStrategy::Any);
        let num_clusters = df::count(clusters)?;

        let trips =
            LazyFrame::scan_parquet("../data/tmp/simplify/trips.parquet", Default::default())?;
        let num_trips = df::count(trips)?;

        Ok(Stats {
            num_stops,
            num_clusters,
            num_trips,
        })
    }

    match collect_stats() {
        Ok(stats) => Ok(web::Json(stats)),
        Err(err) => {
            match err {
                PolarsError::IO { .. } => {
                    // TODO: Use a better way to determine whether service is ready
                    Err(actix_web::Error::from(ErrorInternalServerError(
                        "Unable to read stops_clustered.csv",
                    )))
                }
                _ => Err(actix_web::Error::from(ErrorInternalServerError(""))),
            }
        }
    }
}
