use crate::algorithm::{PreprocessInit, PreprocessingError, PreprocessingInput, PreprocessingResult};
use crate::direct_connections::DirectConnections;
use crate::stp::preprocessing::clustering::filter_for_cluster;
use crate::stp::preprocessing::clustering::k_means::cluster;
use crate::stp::ScalableTransferPatternsAlgorithm;
use crate::tp::transfer_pattern_ds::table::TransferPatternsTable;
use crate::tp::TransferPatternsAlgorithm;
use common::util::df::{write_file, FileType};
use common::util::logging::{run_with_pb, run_with_spinner};
use polars::frame::DataFrame;
use polars::prelude::{col, IntoLazy, JoinArgs};

impl PreprocessInit for ScalableTransferPatternsAlgorithm {
    fn preprocess(input: PreprocessingInput, save_to_disk: bool) -> PreprocessingResult<Self> {
        let (stop_ids_with_clusters, num_clusters) = run_with_spinner("preprocessing", "Clustering stops", || {
            let (stop_ids_with_clusters, num_clusters) = cluster(&input.stops)
                .expect("Clustering failed");

            let stops_clustered = input.stops.clone()
                .left_join(stop_ids_with_clusters.clone().lazy(), "stop_id", "stop_id")
                .collect()?;

            // TODO: Switch to parquet
            write_file("data/tmp/stp/stops_clustered.csv".into(), FileType::CSV, stops_clustered)?;

            Ok::<(DataFrame, u32), PreprocessingError>((stop_ids_with_clusters, num_clusters))
        })?;

        let message = format!("Calculating local transfers for {num_clusters} clusters");
        run_with_pb("preprocessing", message.as_str(), num_clusters as u64, true, |pb| {
            // Currently not parallelized, since individual clusters could take very different amounts
            // of time and RAM usage is lower when only looking at a single cluster at a time.
            // Therefore, we parallelize within one cluster.
            for cluster_id in 0..num_clusters {
                let cluster_result = Self::process_cluster(cluster_id, &stop_ids_with_clusters, &input)?;                
                if save_to_disk {
                    Self::save_cluster(cluster_id, cluster_result)?;
                }
                
                pb.inc(1);
            }

            Ok::<(), PreprocessingError>(())
        })?;

        // TODO
        Ok(Self {})
    }
}

impl ScalableTransferPatternsAlgorithm {
    fn process_cluster(
        cluster_id: u32,
        stop_ids_with_clusters: &DataFrame,
        overall_input: &PreprocessingInput,
    ) -> Result<(TransferPatternsTable, DirectConnections), PreprocessingError> {
        let input =
            filter_for_cluster(cluster_id, stop_ids_with_clusters, overall_input)?;

        write_file(
            format!("./data/tmp/stp/clusters/{cluster_id}/stops.parquet").into(),
            FileType::PARQUET,
            input.stops.clone().collect()?,
        )?;
        write_file(
            format!("./data/tmp/stp/clusters/{cluster_id}/trips.parquet").into(),
            FileType::PARQUET,
            input.trips.clone().collect()?,
        )?;
        write_file(
            format!("./data/tmp/stp/clusters/{cluster_id}/stop_times.parquet").into(),
            FileType::PARQUET,
            input.stop_times.clone().collect()?,
        )?;

        let result = TransferPatternsAlgorithm::preprocess(input.clone(), false)?;

        let TransferPatternsAlgorithm { transfer_patterns, direct_connections } =
            result;

        write_file(
            format!("./data/tmp/stp/clusters/{cluster_id}/transfer_patterns.parquet").into(),
            FileType::PARQUET,
            transfer_patterns.0.clone()
        )?;

        // Build transfer pattern visualization file
        {
            let vis_df = transfer_patterns.0.clone().lazy()
                .join(input.stops.clone(), [col("start")], [col("stop_id")], JoinArgs::default())
                .rename(["lat", "lon"], ["start_lat", "start_lon"], true)
                .join(input.stops.clone(), [col("target")], [col("stop_id")], JoinArgs::default())
                .rename(["lat", "lon"], ["target_lat", "target_lon"], true)
                .drop(["intermediates"]);
            
            println!("{}", vis_df.clone().collect()?);

            write_file(
                format!("./data/tmp/stp/clusters/{cluster_id}/tp_vis.csv").into(),
                FileType::CSV,
                vis_df.collect()?
            )?;
        }

        Ok((transfer_patterns, direct_connections))
    }
    
    fn save_cluster(
        cluster_id: u32,
        (tp_table, direct_connections): (TransferPatternsTable, DirectConnections)
    ) -> Result<(), PreprocessingError> {
        // TODO: Switch to IPC as data format
        
        write_file(
            format!("./data/preprocessing/stp/transfer_patterns/cluster_id={cluster_id}/data.parquet").into(),
            FileType::PARQUET,
            tp_table.0
        )?;

        write_file(
            format!("./data/preprocessing/stp/direct_connections/stop_incidence/cluster_id={cluster_id}/data.parquet").into(),
            FileType::PARQUET,
            direct_connections.stop_incidence
        )?;

        write_file(
            format!("./data/preprocessing/stp/direct_connections/expanded_lines/cluster_id={cluster_id}/data.parquet").into(),
            FileType::PARQUET,
            direct_connections.expanded_lines
        )?;
        
        Ok(())
    }
}