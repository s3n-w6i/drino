use crate::algorithm::{
    PreprocessInit, PreprocessingError, PreprocessingInput, PreprocessingResult,
};
use crate::direct_connections::DirectConnections;
use crate::stp::preprocessing::clustering::filter_for_cluster;
use crate::stp::preprocessing::clustering::k_means::cluster;
use crate::stp::ScalableTransferPatternsAlgorithm;
use crate::tp::transfer_pattern_ds::table::TransferPatternsTable;
use crate::tp::TransferPatternsAlgorithm;
use arrow_array::UInt32Array;
use arrow_schema::{DataType, Field};
use common::util::df::{write_df_to_file, write_geoarrow_to_file, FileType};
use common::util::geoarrow_lines::build_geoarrow_lines;
use common::util::logging::{run_with_pb, run_with_spinner};
use polars::frame::DataFrame;
use polars::prelude::IntoLazy;
use std::sync::Arc;

impl PreprocessInit for ScalableTransferPatternsAlgorithm {
    fn preprocess(input: PreprocessingInput, save_to_disk: bool) -> PreprocessingResult<Self> {
        let (stop_ids_with_clusters, num_clusters) =
            run_with_spinner("preprocessing", "Clustering stops", || {
                let (stop_ids_with_clusters, num_clusters) =
                    cluster(&input.stops).expect("Clustering failed");

            let stops_clustered = input.stops.clone()
                .left_join(stop_ids_with_clusters.clone().lazy(), "stop_id", "stop_id")
                .collect()?;

                // TODO: Switch to parquet
                write_df_to_file(
                    "data/tmp/stp/stops_clustered.csv".into(),
                    FileType::CSV,
                    stops_clustered,
                )?;

                Ok::<(DataFrame, u32), PreprocessingError>((stop_ids_with_clusters, num_clusters))
            })?;

        let message = format!("Calculating local transfers for {num_clusters} clusters");
        run_with_pb("preprocessing", message.as_str(), num_clusters as u64, true, |pb| {
            // Currently not parallelized, since individual clusters could take very different amounts
            // of time and RAM usage is lower when only looking at a single cluster at a time.
            // Therefore, we parallelize within one cluster.
            for cluster_id in 0..num_clusters {
                let cluster_result =
                    Self::process_cluster(cluster_id, &stop_ids_with_clusters, &input)?;
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
        let input = filter_for_cluster(cluster_id, stop_ids_with_clusters, overall_input)?;

        write_df_to_file(
            format!("./data/tmp/stp/clusters/{cluster_id}/stops.parquet").into(),
            FileType::PARQUET,
            input.stops.clone().collect()?,
        )?;
        write_df_to_file(
            format!("./data/tmp/stp/clusters/{cluster_id}/trips.parquet").into(),
            FileType::PARQUET,
            input.trips.clone().collect()?,
        )?;
        write_df_to_file(
            format!("./data/tmp/stp/clusters/{cluster_id}/stop_times.parquet").into(),
            FileType::PARQUET,
            input.stop_times.clone().collect()?,
        )?;

        let result = TransferPatternsAlgorithm::preprocess(input.clone(), false)?;

        let TransferPatternsAlgorithm { transfer_patterns, direct_connections } = result;

        // Build transfer patterns visualization
        {
            let stop_chains = transfer_patterns.0.iter()
                .map(|tp| [vec![tp.0], tp.1.clone(), vec![tp.2]].concat());

            let mut table = build_geoarrow_lines(
                stop_chains.collect(),
                input.stops.clone(),
            )?;

            let start_field = Field::new("start", DataType::UInt32, false);
            let target_field = Field::new("target", DataType::UInt32, false);
            let start_id_array = UInt32Array::from_iter(
                transfer_patterns.0.iter().map(|tp| tp.0.0)
            );
            let target_id_array = UInt32Array::from_iter(
                transfer_patterns.0.iter().map(|tp| tp.2.0)
            );
            table.append_column(start_field.into(), vec![Arc::new(start_id_array)])?;
            table.append_column(target_field.into(), vec![Arc::new(target_id_array)])?;

            write_geoarrow_to_file(
                format!("./data/tmp/stp/clusters/{cluster_id}/transfer_patterns.arrow").into(),
                FileType::IPC,
                table,
            )?;
        }

        // Build lines visualization
        {
            let table = direct_connections.to_geoarrow_lines(input.stops)?;

            write_geoarrow_to_file(
                format!("./data/tmp/stp/clusters/{cluster_id}/lines_geo.arrow").into(),
                FileType::IPC,
                table,
            )?;
        }

        Ok((transfer_patterns, direct_connections))
    }

    fn save_cluster(
        cluster_id: u32,
        (tp_table, direct_connections): (TransferPatternsTable, DirectConnections),
    ) -> Result<(), PreprocessingError> {
        // TODO: Switch to IPC as data format

        /* TODO: write_file(
            format!("./data/preprocessing/stp/transfer_patterns/cluster_id={cluster_id}/data.parquet").into(),
            FileType::PARQUET,
            tp_table.0
        )?; */

        write_df_to_file(
            format!("./data/preprocessing/stp/direct_connections/stop_incidence/cluster_id={cluster_id}/data.parquet").into(),
            FileType::PARQUET,
            direct_connections.stop_incidence
        )?;

        write_df_to_file(
            format!("./data/preprocessing/stp/direct_connections/expanded_lines/cluster_id={cluster_id}/data.parquet").into(),
            FileType::PARQUET,
            direct_connections.expanded_lines
        )?;

        Ok(())
    }
}
