use std::fmt;
use std::fmt::Display;

use linfa::traits::Transformer;
use linfa_clustering::Dbscan;
use polars::io::SerWriter;
use polars::prelude::{col, CsvWriter, Float32Type, IndexOrder, LazyFrame, Literal};
use polars::series::Series;

const MIN_CLUSTER_SIZE: usize = 500;

pub async fn cluster(
    stops: &LazyFrame,
) -> Result<(), DbscanClusterError> {
    let stops_array = stops.clone()
        .select([col("lat"), col("lon")])
        .collect()?
        .to_ndarray::<Float32Type>(IndexOrder::default())?;
    let contiguous_stops_array = stops_array.as_standard_layout();

    let clusters = Dbscan::params(MIN_CLUSTER_SIZE)
        .tolerance(50e-2)
        .transform(&contiguous_stops_array)
        .unwrap();

    let cluster_series: Series = clusters.into_iter()
        .map(|x| match x {
            Some(x) => {x as i32}
            None => -1
        })
        .collect::<Series>()
        .with_name("cluster_id".into());

    println!("{:?}", cluster_series);

    let mut stop_ids_with_clusters = stops.clone()
        //.select([ col("stop_id") ])
        .with_column(cluster_series.lit())
        .collect()?;

    let mut file = std::fs::File::create("../../../../stops_clustered.csv").unwrap();
    CsvWriter::new(&mut file).finish(&mut stop_ids_with_clusters).unwrap();


    //Ok(stop_ids_with_clusters)
    Ok(())
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