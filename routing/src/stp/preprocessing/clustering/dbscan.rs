use std::fmt;
use std::fmt::Display;

use linfa::traits::Transformer;
use linfa_clustering::Dbscan;
use polars::frame::DataFrame;
use polars::prelude::{col, Float32Type, IndexOrder, LazyFrame, Literal};
use polars::series::Series;

const MIN_CLUSTER_SIZE: usize = 500;

pub fn cluster(
    stops: &LazyFrame,
) -> Result<DataFrame, DbscanClusterError> {
    let stops_array = stops.clone()
        .select([col("lat"), col("lon")])
        .collect()?
        .to_ndarray::<Float32Type>(IndexOrder::default())?;
    let contiguous_stops_array = stops_array.as_standard_layout();

    let clusters = Dbscan::params(MIN_CLUSTER_SIZE)
        .tolerance(50e-2)
        .transform(&contiguous_stops_array)?;

    let cluster_series: Series = clusters.into_iter()
        .map(|x| match x {
            Some(x) => {x as i32}
            None => -1
        })
        .collect::<Series>()
        .with_name("cluster_id".into());
    
    let stop_ids_with_clusters = stops.clone()
        .with_column(cluster_series.lit())
        .collect()?;

    Ok(stop_ids_with_clusters)
}

#[derive(thiserror::Error, Debug)]
pub enum DbscanClusterError {
    Polars(#[from] polars::error::PolarsError),
    Dbscan(#[from] linfa_clustering::DbscanParamsError),
}

impl Display for DbscanClusterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err: &dyn Display = match self {
            DbscanClusterError::Polars(err) => err,
            DbscanClusterError::Dbscan(err) => err,
        };
        write!(f, "{}", err)
    }
}