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
use polars::frame::{DataFrame, UniqueKeepStrategy};
use polars::prelude::{col, lit, Column, IntoLazy, LazyFrame};
use std::sync::Arc;
use geo::{coord, point, Distance, Haversine};
use log::debug;

// The minimum average distance between stations for a line to be considered long-distance. In
// meters.
const LONG_DISTANCE_AVG_DISTANCE: u32 = 10_000;

impl PreprocessInit for ScalableTransferPatternsAlgorithm {
    fn preprocess(input: PreprocessingInput, save_to_disk: bool) -> PreprocessingResult<Self> {
        let direct_connections = run_with_spinner("preprocessing", "Calculating direct connections", || {
            let direct_connections = DirectConnections::try_from(input.clone())?;
            debug!(target: "preprocessing", "Direct connections built");

            // Build visualization
            let geoarrow_table = direct_connections
                .to_geoarrow_lines(input.stops.clone())
                .map_err(|e| PreprocessingError::BuildLines(e))?;

            write_geoarrow_to_file("./data/tmp/global/lines.arrow".into(), FileType::IPC, geoarrow_table)
                .map_err(|e| PreprocessingError::GeoArrow(e))?;
            debug!(target: "preprocessing", "Geo-Arrow table of direct connections written");

            Ok::<DirectConnections, PreprocessingError>(direct_connections)
        })?;

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

        let long_distance_stations =
            run_with_spinner("preprocessing", "Finding long-distance stations", || {
                Self::find_long_distance_stations(direct_connections, input.stops)
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
    
    fn find_long_distance_stations(direct_connections: DirectConnections, stops: LazyFrame) -> Result<LazyFrame, PreprocessingError> {
        // Add lat and lon from stops df to the lines df
        let mut lines_with_coordinates = direct_connections.line_progressions.lazy()
            .inner_join(stops, col("stop_id"), col("stop_id"))
            .collect()?;

        // Calculate the distance to the previous station for each station on a line
        let line_ids = lines_with_coordinates.column("line_id")?.u32()?;
        let lats = lines_with_coordinates.column("lat")?.f32()?;
        let lons = lines_with_coordinates.column("lon")?.f32()?;
        let mut distances_to_prev = Vec::with_capacity(line_ids.len());

        distances_to_prev.push(None); // Push an initial None value for the first entry
        for i in 1..line_ids.len() {
            let prev_line_id = &line_ids.get(i - 1).unwrap();
            let curr_line_id = &line_ids.get(i).unwrap();
            // Check if we are at the start of a new line, where the prev station is from
            // another line. In this case, do not calculate a distance.
            let value = if curr_line_id != prev_line_id {
                None
            } else {
                let lat_window = &lats.slice((i - 1) as i64, 2);
                let lon_window = &lons.slice((i - 1) as i64, 2);

                let previous_point = point!(coord! { x: lat_window.get(0).unwrap(), y: lon_window.get(0).unwrap() });
                let current_point = point!(coord! { x: lat_window.get(1).unwrap(), y: lon_window.get(1).unwrap() });

                let distance = Haversine::distance(previous_point, current_point);
                Some(distance)
            };
            distances_to_prev.push(value);
        }

        let lines_with_distances = lines_with_coordinates
            .with_column(Column::new("distance_to_previous".into(), distances_to_prev))?;

        // Calculate the average of all distances in the line. Drop useless columns.
        let lines_with_average_distance = lines_with_distances.clone()
            .lazy()
            .select([
                col("stop_id"),
                col("distance_to_previous")
                    .mean()
                    .over(["line_id"])
                    .alias("average_distance"),
            ]);
        
        // Keep the long-distance stations
        let long_distance_stations = lines_with_average_distance
            .filter(col("average_distance").gt_eq(lit(LONG_DISTANCE_AVG_DISTANCE)))
            .select([col("stop_id")])
            .unique(None, UniqueKeepStrategy::Any);

        Ok(long_distance_stations)
    }
}
