use linfa::prelude::{Fit, Predict};
use linfa::DatasetBase;
use linfa_clustering::KMeans;
use polars::frame::DataFrame;
use polars::prelude::{col, Float32Type, IndexOrder, LazyFrame, Literal};
use polars::series::Series;
use std::fmt;
use std::fmt::Display;

const NUM_CLUSTERS: u32 = 8;

pub fn cluster(
    stops: &LazyFrame,
) -> Result<(DataFrame, u32), KmeansClusterError> {
    let stops_array = stops.clone()
        .select([ col("lat"), col("lon")])
        .collect()?
        .to_ndarray::<Float32Type>(IndexOrder::default())?;
    let stops_data = DatasetBase::from(stops_array.as_standard_layout().clone());

    let k_means_model = KMeans::params(NUM_CLUSTERS as usize)
        .fit(&stops_data)?;
    let result = k_means_model.predict(stops_array);

    let cluster_id_series: Series = result.targets.into_iter()
        .map(|x| x as u32)
        .collect::<Series>()
        .with_name("cluster_id".into());

    let stop_ids_with_clusters = stops.clone()
        .select([ col("stop_id") ])
        .with_column(cluster_id_series.lit())
        .collect()?;

    Ok((stop_ids_with_clusters, NUM_CLUSTERS))
}

#[derive(thiserror::Error, Debug)]
pub enum KmeansClusterError {
    Polars(#[from] polars::error::PolarsError),
    KMeans(#[from] linfa_clustering::KMeansError),
}

impl Display for KmeansClusterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err: &dyn Display = match self {
            KmeansClusterError::Polars(err) => err,
            KmeansClusterError::KMeans(err) => err,
        };
        write!(f, "{}", err)
    }
}