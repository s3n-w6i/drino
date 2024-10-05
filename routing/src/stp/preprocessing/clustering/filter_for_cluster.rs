use polars::frame::UniqueKeepStrategy;
use polars::prelude::*;

use crate::algorithm::{PreprocessingError, PreprocessingInput};

pub fn filter_for_cluster(
    cluster_id: u32,
    // columns: "stop_id", "cluster_id"
    stop_ids_with_cluster_ids: LazyFrame,
    PreprocessingInput {
        stops, stop_times, trips, services
    }: &PreprocessingInput,
) -> Result<PreprocessingInput, PreprocessingError> {
    let stop_ids_in_this_cluster = stop_ids_with_cluster_ids
        .filter(col("cluster_id").eq(lit(cluster_id)))
        .select([col("stop_id")])
        // We want to reassign new ids, so that they are continuous again
        .rename(["stop_id"], ["pre_cluster_stop_id"])
        .with_row_index("new_stop_id", None);

    // Filter the stops
    let stops = stops.clone()
        .join(
            stop_ids_in_this_cluster.clone(),
            [col("stop_id")],
            [col("pre_cluster_stop_id")],
            JoinArgs::new(
                JoinType::Inner
            )
        )
        .rename(["stop_id"], ["pre_cluster_stop_id"]);

    // Only include stop times that are within the cluster
    // Since lines (in RAPTOR) will be calculated based only on the stop_times-table, resulting
    // lines will "skip over" the parts of a line that are outside the cluster. This is fine, since
    // we don't care about what happens outside of this cluster.
    let stop_times = stop_times.clone()
        // Only keep stops that are in the cluster
        .inner_join(
            stop_ids_in_this_cluster.clone(),
            col("stop_id"),
            col("pre_cluster_stop_id"),
        )
        // don't keep the original stop id...
        .drop(["stop_id"])
        // ...instead replace it with the new one
        .rename(["new_stop_id"], ["stop_id"]);
    
    let trip_ids_in_this_cluster = stop_times.clone()
        .select([col("trip_id")])
        .unique(None, UniqueKeepStrategy::Any);

    let trips = trips.clone()
        .semi_join(
            trip_ids_in_this_cluster,
            col("trip_id"),
            col("trip_id"),
        );

    let service_ids_in_this_cluster = trips.clone()
        .select([col("service_id")])
        .unique(None, UniqueKeepStrategy::Any);

    let services = services.clone()
        .semi_join(
            service_ids_in_this_cluster,
            col("service_id"),
            col("service_id"),
        );
    
    let stops = stops
        // From now on, use the new stop id as the regular stop id
        .rename(["new_stop_id"], ["stop_id"]);

    Ok(PreprocessingInput {
        services: services.clone(),
        stops,
        trips,
        stop_times,
    })
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_filter_for_cluster() {
        let stop_ids_with_clusters = df!(
            "stop_id"    => &[0u32, 1, 2, 3, 4, 5],
            "cluster_id" => &[0u32, 0, 1, 1, 2, 2],
        ).unwrap().lazy();
        let stops = df!(
            "stop_id"    => &[0u32, 1, 2, 3, 4, 5]
        ).unwrap().lazy();
        let stop_times = df!(
            "stop_id" => [0u32, 0, 0, 1, 2, 2, 3, 4, 5],
            "trip_id" => [0u32, 1, 2, 3, 4, 5, 6, 7, 8]
        ).unwrap().lazy();
        let trips = df!(
            "trip_id"    => [0u32, 1, 2, 3, 4, 5, 6, 7, 8],
            "service_id" => [0u32, 1, 2, 3, 4, 5, 6, 7, 8],
        ).unwrap().lazy();
        let services = df!(
            "service_id" => [0u32, 1, 2, 3, 4, 5, 6, 7, 8],
        ).unwrap().lazy();

        let PreprocessingInput {
            stops: filtered_stops,
            stop_times: filtered_stop_times,
            trips: filtered_trips,
            services: filtered_services,
        } = filter_for_cluster(
            1,
            stop_ids_with_clusters,
            &PreprocessingInput { stops, stop_times, trips, services },
        ).unwrap();

        let filtered_stops_ids = filtered_stops.collect().unwrap()
            .column("stop_id").unwrap()
            .u32().unwrap()
            .to_vec();
        assert_eq!(filtered_stops_ids.len(), 2);
        // The new stop ids are continuous starting from 0
        assert!(filtered_stops_ids.contains(&Some(0)));
        assert!(filtered_stops_ids.contains(&Some(1)));

        let filtered_stop_times_station_ids = filtered_stop_times.clone().collect().unwrap()
            .column("stop_id").unwrap()
            .u32().unwrap()
            .to_vec();
        assert_eq!(filtered_stop_times_station_ids.len(), 3);
        assert!(filtered_stop_times_station_ids.contains(&Some(0)));
        assert!(filtered_stop_times_station_ids.contains(&Some(1)));
        let filtered_stop_times_trip_ids = filtered_stop_times.collect().unwrap()
            .column("trip_id").unwrap()
            .u32().unwrap()
            .to_vec();
        assert_eq!(filtered_stop_times_trip_ids.len(), 3);
        assert!(filtered_stop_times_trip_ids.contains(&Some(4)));
        assert!(filtered_stop_times_trip_ids.contains(&Some(5)));
        assert!(filtered_stop_times_trip_ids.contains(&Some(6)));

        let filtered_trip_ids = filtered_trips.collect().unwrap()
            .column("trip_id").unwrap()
            .u32().unwrap()
            .to_vec();
        assert_eq!(filtered_trip_ids.len(), 3);
        assert!(filtered_trip_ids.contains(&Some(4)));
        assert!(filtered_trip_ids.contains(&Some(5)));
        assert!(filtered_trip_ids.contains(&Some(6)));

        let filtered_service_ids = filtered_services.collect().unwrap()
            .column("service_id").unwrap()
            .u32().unwrap()
            .to_vec();
        assert_eq!(filtered_service_ids.len(), 3);
        assert!(filtered_service_ids.contains(&Some(4)));
        assert!(filtered_service_ids.contains(&Some(5)));
        assert!(filtered_service_ids.contains(&Some(6)));
    }
}