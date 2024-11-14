use crate::algorithm::{PreprocessInit, PreprocessingError, PreprocessingInput, PreprocessingResult};
use crate::stp::preprocessing::clustering::filter_for_cluster;
use crate::stp::preprocessing::clustering::k_means::cluster;
use crate::stp::ScalableTransferPatternsAlgorithm;
use crate::tp::TransferPatternsAlgorithm;
use crate::write_tmp_file;
use common::util::logging::{run_with_pb, run_with_spinner};
use polars::frame::DataFrame;
use polars::io::SerWriter;
use polars::prelude::{CsvWriter, IntoLazy, LazyFrame};

impl PreprocessInit for ScalableTransferPatternsAlgorithm {
    fn preprocess(input: PreprocessingInput) -> PreprocessingResult<Self> {
        let (stop_ids_with_clusters, num_clusters) = run_with_spinner("preprocessing", "Clustering stops", || {
            let (stop_ids_with_clusters, num_clusters) = cluster(&input.stops)
                .expect("Clustering failed");

            let mut stops_clustered = input.stops.clone()
                .left_join(stop_ids_with_clusters.clone().lazy(), "stop_id", "stop_id")
                .collect()?;

            // TODO: Switch to parquet and write_tmp_file
            let mut file = std::fs::File::create("./data/tmp/stp/stops_clustered.csv").unwrap();
            CsvWriter::new(&mut file).finish(&mut stops_clustered).unwrap();

            Ok::<(DataFrame, u32), PreprocessingError>((stop_ids_with_clusters, num_clusters))
        })?;

        let stop_ids_with_clusters = stop_ids_with_clusters.lazy();
        
        let message = format!("Calculating local transfers for {num_clusters} clusters");
        run_with_pb("preprocessing", message.as_str(), num_clusters as u64, true, |pb| {
            // Currently not parallelized, since individual clusters could take very different amounts
            // of time and RAM usage is lower when only looking at a single cluster at a time.
            // Therefore, we parallelize within one cluster.
            for cluster_id in 0..num_clusters {
                pb.inc(1);
                process_cluster(cluster_id, &stop_ids_with_clusters, &input)?;
            }

            Ok::<(), PreprocessingError>(())
        })?;

        // TODO
        Ok(Self {})
    }
}


fn process_cluster(
    cluster_id: u32,
    stop_ids_with_clusters: &LazyFrame,
    overall_input: &PreprocessingInput,
) -> Result<(), PreprocessingError> {
    let (cluster_filtered_input, stop_id_mapping) =
        filter_for_cluster(cluster_id, &stop_ids_with_clusters, &overall_input)?;

    write_tmp_file(
        format!("./data/tmp/stp/clusters/{cluster_id}/stops.parquet").into(),
        &mut cluster_filtered_input.stops.clone().collect()?
    )?;
    write_tmp_file(
        format!("./data/tmp/stp/clusters/{cluster_id}/trips.parquet").into(),
        &mut cluster_filtered_input.trips.clone().collect()?
    )?;
    write_tmp_file(
        format!("./data/tmp/stp/clusters/{cluster_id}/stop_times.parquet").into(),
        &mut cluster_filtered_input.stop_times.clone().collect()?
    )?;

    let mut cluster_result = TransferPatternsAlgorithm::preprocess(cluster_filtered_input)?;

    cluster_result.rename_stops(stop_id_mapping)?;

    drop(cluster_result);

    Ok(())
}