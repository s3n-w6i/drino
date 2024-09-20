use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};
use log::info;
use polars::frame::DataFrame;
use polars::io::SerWriter;
use polars::prelude::{CsvWriter, IntoLazy};
use common::util::logging::run_with_spinner;
use crate::algorithm::{PreprocessInit, PreprocessingError, PreprocessingInput, PreprocessingResult};
use crate::stp::preprocessing::clustering::filter_for_cluster;
use crate::stp::preprocessing::clustering::gmm::cluster;
use crate::stp::ScalableTransferPatternsAlgorithm;
use crate::tp::TransferPatternsAlgorithm;

impl PreprocessInit for ScalableTransferPatternsAlgorithm {
    fn preprocess(input: PreprocessingInput) -> PreprocessingResult<Self> {
        let (stop_ids_with_clusters, num_clusters) = run_with_spinner("preprocessing", "Clustering stops", || {
            let (stop_ids_with_clusters, num_clusters) = cluster(&input.stops)
                .expect("Clustering failed");

            let mut stops_clustered = input.stops.clone()
                .left_join(stop_ids_with_clusters.clone().lazy(), "stop_id", "stop_id")
                .collect()?;

            let mut file = std::fs::File::create("./data/tmp/stp/stops_clustered.csv").unwrap();
            CsvWriter::new(&mut file).finish(&mut stops_clustered).unwrap();

            Ok::<(DataFrame, u32), PreprocessingError>((stop_ids_with_clusters, num_clusters))
        })?;

        let cluster_processing_pb = ProgressBar::new(num_clusters as u64)
            .with_message("Processing clusters...")
            .with_style(ProgressStyle::with_template("[{elapsed}] {msg} {wide_bar} {human_pos}/{human_len} eta: {eta}").unwrap());

        // Currently not parallelized, since individual clusters could take very different amounts
        // of time. Therefore, we aim to parallelize within one cluster.
        for cluster_id in (0..num_clusters).progress_with(cluster_processing_pb) {
            let cluster_filtered_input = filter_for_cluster(cluster_id, stop_ids_with_clusters.clone().lazy(), &input)
                .expect("todo");
            let cluster_result = TransferPatternsAlgorithm::preprocess(cluster_filtered_input)?;

            // TODO: Transform stop ids back to original values after transfer pattern calculation

            drop(cluster_result);
        }

        info!(target: "preprocessing", "Cluster processing finished ({num_clusters} clusters)");

        // TODO
        Ok(Self { })
    }
}