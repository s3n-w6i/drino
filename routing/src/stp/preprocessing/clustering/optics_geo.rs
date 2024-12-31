use std::fmt;
use std::fmt::Display;

use geo::{point, Distance, Haversine};
use linfa::Float;
use linfa::traits::Transformer;
use linfa_clustering::{Optics, OpticsAnalysis, Sample};
use linfa_nn::{BallTreeIndex, CommonNearestNeighbour, NearestNeighbourIndex};
use ndarray::{ArrayBase, ArrayView, CowRepr, Data, Dimension, Ix2};
use polars::frame::{DataFrame, UniqueKeepStrategy};
use polars::prelude::*;

const MIN_POINTS: usize = 8;
const MIN_CLUSTER_SIZE: u32 = 300;
const MAX_CLUSTER_SIZE: u32 = 1_500;

#[derive(Debug, Clone, PartialEq, Eq)]
struct HaversineDist;

impl<F: Float> linfa_nn::distance::Distance<F> for HaversineDist {
    fn distance<D: Dimension>(&self, a: ArrayView<F, D>, b: ArrayView<F, D>) -> F {
        let mut a_iter = a.into_iter();
        let mut b_iter = b.into_iter();
        let point_a = point!(x: *a_iter.next().unwrap(), y: *a_iter.next().unwrap());
        let point_b = point!(x: *b_iter.next().unwrap(), y: *b_iter.next().unwrap());
        let distance = Haversine::distance(point_a, point_b);
        F::from(distance).unwrap()
    }
}

pub fn cluster(
    stops: &LazyFrame,
) -> Result<(DataFrame, u32), OpticsClusterError> {
    let stops_array = stops.clone()
        .select([col("lat"), col("lon")])
        .collect()?
        .to_ndarray::<Float32Type>(IndexOrder::default())?;
    let contiguous_stops_array = stops_array.as_standard_layout();
    //let stops_data = DatasetBase::from(stops_array.as_standard_layout().clone());

    let analysis = Optics::params_with(
        MIN_POINTS,
        HaversineDist,
        CommonNearestNeighbour::KdTree,
    ).tolerance(2_000.0) // radius in meters (should be as small as possible for best runtime)
        .transform(contiguous_stops_array.view())?;


    let clusters = extract_clusters(contiguous_stops_array.clone(), &analysis, 300.0)?;
    let num_clusters = clusters.clone().lazy()
        .unique(Some(vec![String::from("cluster_id")]), UniqueKeepStrategy::Any)
        .select([len()])
        .collect()?.to_ndarray::<UInt32Type>(IndexOrder::default())?.row(0)[0];

    let stops_clustered = stops.clone()
        .select([col("stop_id")])
        // TODO: Replace with a stable id?
        .with_row_index("index", None)
        .left_join(clusters.lazy(), "index", "stop_index")
        .drop(["index"])
        .collect()?;

    Ok((stops_clustered, num_clusters))
}

fn extract_clusters(
    data: ArrayBase<CowRepr<f32>, Ix2>, analysis: &OpticsAnalysis<f32>, eps: f32,
) -> Result<DataFrame, OpticsClusterError> {
    let index = BallTreeIndex::new(&data, 4, HaversineDist)
        .expect("failed to construct ball tree index");

    let mut clusters_cluster_column: Vec<u32> = vec![];
    let mut clusters_stop_column: Vec<u32> = vec![];
    let mut current_cluster_id: u32 = 0;
    let mut current_cluster_size: u32 = 0;

    // TODO: par_bridge
    analysis.iter().try_for_each(|sample| -> Result<(), OpticsClusterError> {
        if sample.reachability_distance().unwrap_or(f32::INFINITY) <= eps {
            if clusters_cluster_column.is_empty() {
                // Make a new cluster if there is none already
                current_cluster_id = 1;
            }
            clusters_cluster_column.push(current_cluster_id - 1); // Ok, since cluster id is 1 or more here
            clusters_stop_column.push(sample.index() as u32);
            current_cluster_size += 1;
        } else {
            let n = build_neighbourhood(&data, &index, sample, eps)?;
            if n.len() >= MIN_POINTS && sample.core_distance().unwrap_or(f32::INFINITY) <= eps {
                clusters_cluster_column.push(current_cluster_id);
                clusters_stop_column.push(sample.index() as u32);
                current_cluster_size += 1;
                // Complete this cluster and make a new one (if big enough)
                // Todo: This is a pretty dumb way to do this, improve it (evaluate if better to merge with previous or next etc.)
                if current_cluster_size >= MIN_CLUSTER_SIZE {
                    current_cluster_id += 1;
                    current_cluster_size = 0;
                }
            } else {
                if clusters_cluster_column.is_empty() {
                    current_cluster_id = 1;
                }
                debug_assert!(current_cluster_id >= 1);
                clusters_cluster_column.push(current_cluster_id - 1); // Ok, since cluster id is 1 or more here
                clusters_stop_column.push(sample.index() as u32);
                current_cluster_size += 1;
            }
        }
        Ok(())
    })?;

    let clusters = DataFrame::new(vec![
        Column::new("cluster_id".into(), &clusters_cluster_column),
        Column::new("stop_index".into(), &clusters_stop_column),
    ])?;

    Ok(clusters)
}

fn build_neighbourhood<D: Data<Elem=f32>>(data: &ArrayBase<D, Ix2>, index: &BallTreeIndex<f32, HaversineDist>, sample: &Sample<f32>, tolerance: f32) -> Result<Vec<usize>, OpticsClusterError> {
    let neighbors = index.within_range(data.row(sample.index()), tolerance)?;

    Ok(neighbors.into_iter()
        .map(|(_, idx)| idx)
        .collect())
}


#[derive(thiserror::Error, Debug)]
pub enum OpticsClusterError {
    Polars(#[from] PolarsError),
    Optics(#[from] linfa_clustering::OpticsError),
    NN(#[from] linfa_nn::NnError),
}

impl Display for OpticsClusterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err: &dyn Display = match self {
            OpticsClusterError::Polars(err) => err,
            OpticsClusterError::Optics(err) => err,
            OpticsClusterError::NN(err) => err,
        };
        write!(f, "{}", err)
    }
}