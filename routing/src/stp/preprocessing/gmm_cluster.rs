use std::fmt;
use std::fmt::Display;

use linfa::DatasetBase;
use linfa::traits::{Fit, Predict};
use linfa_clustering::GaussianMixtureModel;
use polars::frame::DataFrame;
use polars::io::SerWriter;
use polars::prelude::{col, CsvWriter, Float32Type, IndexOrder, LazyFrame, Literal};
use polars::series::Series;

pub async fn cluster(
    stops: &LazyFrame,
) -> Result<(DataFrame, usize), GmmClusterError> {
    let stops_array = stops.clone()
        .select([col("lat"), col("lon")])
        .collect()?
        .to_ndarray::<Float32Type>(IndexOrder::default())?;
    let stops_data = DatasetBase::from(stops_array.clone());

    // determine the number of clusters roughly by looking at the numer of stops
    let num_clusters = (stops_array.shape()[0] as f32 / 850.0).round() as usize;

    let model = GaussianMixtureModel::params(num_clusters)
        .n_runs(10)
        .max_n_iterations(100)
        .tolerance(1e-4)
        .fit(&stops_data)
        .expect("GMM fitting failed");
    let result = model.predict(stops_array);

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


    Ok((stop_ids_with_clusters, num_clusters))
}

#[derive(thiserror::Error, Debug)]
pub enum GmmClusterError {
    Polars(#[from] polars::error::PolarsError),
    Gmm(#[from] linfa_clustering::GmmError),
}

impl Display for GmmClusterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err: &dyn Display = match self {
            GmmClusterError::Polars(err) => err,
            GmmClusterError::Gmm(err) => err,
        };
        write!(f, "{}", err)
    }
}