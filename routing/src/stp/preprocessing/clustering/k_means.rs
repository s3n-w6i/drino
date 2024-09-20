use std::fmt;
use std::fmt::Display;
use linfa::DatasetBase;
use linfa::prelude::{Fit, Predict};
use linfa_clustering::KMeans;
use polars::frame::DataFrame;
use polars::io::SerWriter;
use polars::prelude::{col, CsvWriter, Float32Type, IndexOrder, LazyFrame, Literal};
use polars::series::Series;

const NUM_CLUSTERS: usize = 8;

pub async fn cluster(
    stops: &LazyFrame,
) -> Result<DataFrame, KmeansClusterError> {
    let stops_array = stops.clone()
        .select([ col("lat"), col("lon")])
        .collect()?
        .to_ndarray::<Float32Type>(IndexOrder::default())?;
    let stops_data = DatasetBase::from(stops_array.as_standard_layout().clone());

    let k_means_model = KMeans::params(NUM_CLUSTERS)
        .fit(&stops_data)?;
    let result = k_means_model.predict(stops_array);

    let target_series: Series = result.targets.into_iter()
        .map(|x| x as u32)
        .collect::<Series>()
        .with_name("cluster_id".into());

    let mut stop_ids_with_clusters = stops.clone()
        //.select([ col("stop_id") ])
        .with_column(target_series.lit())
        .collect()?;

    let mut file = std::fs::File::create("../../../../stops_clustered.csv").unwrap();
    CsvWriter::new(&mut file).finish(&mut stop_ids_with_clusters).unwrap();

    Ok(stop_ids_with_clusters)
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