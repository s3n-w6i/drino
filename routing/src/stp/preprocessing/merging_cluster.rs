use std::collections::hash_map::RandomState;
use std::time::SystemTime;
use dashmap::{DashMap, DashSet};
use ordered_float::OrderedFloat;
use polars::prelude::*;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};

// Implements merge-based clustering as described in section 3.1.2 of "Scalable Transfer Patterns"

const MAX_CLUSTER_SIZE: u32 = 1_500;

type Map<K, V, S = RandomState> = DashMap<K, V, S>;

// FIXME: Cluster on stop areas, rather than on individual stops (if areas are usually merged)

pub async fn cluster(
    services: &LazyFrame,
    stops: &LazyFrame,
    stop_times: &LazyFrame,
    trips: &LazyFrame,
) -> Result<(), PolarsError> {
    let stop_times = stop_times.clone()
        // Only keep what we need for clustering
        .select([
            col("trip_id"), col("stop_sequence"), col("stop_id")
        ]);

    // See original paper linked above for explanation
    let weight_function =
        (lit(1) / col("s(u)"))
        * (lit(1) / col("s(v)"))
        // TODO: Can we optimize the sqrt?
        * ((col("w(u,v)") / col("s(u)").sqrt()) + (col("w(u,v)") / col("s(v)").sqrt()));


    // A table of how each stop is connected to a following stop by some trip. Graph is directed.
    let stops_adjacency = stop_times.clone()
        .join(
            stop_times.clone(),
            // Add one to the stop sequence number to select the next stop in the sequence
            &[col("trip_id"), col("stop_sequence") + lit(1)],
            &[col("trip_id"), col("stop_sequence")],
            JoinArgs::new(JoinType::Left),
        )
        // The last entry per trip has no following stop. It will be null, so drop it.
        .drop_nulls(Some(vec![col("stop_id_right")]))
        // Keep only relevant columns and rename them.
        .select([
            col("stop_id").alias("from_stop_id"),
            col("stop_id_right").alias("to_stop_id"),
            col("trip_id")
        ]);

    let weighted_stops_adjacency = stops_adjacency
        .join(
            trips.clone(),
            [col("trip_id")],
            [col("trip_id")],
            JoinArgs::new(JoinType::Inner)
        )
        .select([
            col("from_stop_id"), col("to_stop_id"),
            col("service_id")
        ])
        .join(
            services.clone().select([
                // Calculate how many times in a year this service runs
                // TODO: Handle exceptions in calendar_dates
                // TODO: Support definition of journey times only by calender_dates
                // TODO: Handle start and end times of services, since frequent service changes are currently weighed too strongly
                // Sum all days in the week, where this service runs:
                (fold_exprs(
                    lit(0),                      // Start at zero, ...
                    |acc, x| (acc + x).map(Some),  // ...then add all days in the week
                    [col("monday"), col("tuesday"), col("wednesday"), col("thursday"), col("friday"), col("saturday"), col("sunday")]
                ) * lit(52)) // weeks in a year
                    .cast(DataType::UInt32) // amounts to max. 136 trips per second
                    .alias("trips_per_year"),
                col("service_id")
            ]),
            [col("service_id")],
            [col("service_id")],
            JoinArgs::new(JoinType::Inner)
        )
        // Sum up number of trips from identical from-to-pairs
        .group_by([ col("from_stop_id"), col("to_stop_id") ])
        .agg([ sum("trips_per_year").alias("trips_per_year") ])
        .collect()?;

    let stop_count = {
        let count = stops.clone().count()
            .collect()?;
        match count[0].get(0)? {
            AnyValue::UInt32(count) => count,
            _ => panic!("Count wasn't u32")
        }
    };
    dbg!(&stop_count);

    // Keep track of what clusters are adjacent
    let cluster_adjacency: Map<u32, DashSet<(u32, u32)>> = Map::with_capacity(stop_count as usize);

    let adjacency_count = weighted_stops_adjacency[0].len();
    (0..adjacency_count).into_par_iter()
        .for_each(|index| {
            let row = &weighted_stops_adjacency.get_row(index).unwrap().0;
            let from_cluster_id =
                if let AnyValue::UInt32(int) = row.get(0).unwrap() {
                    *int
                } else { panic!("Wrong data type")};
            let to_cluster_id =
                if let AnyValue::UInt32(int) = row.get(1).unwrap() {
                    *int
                } else { panic!("Wrong data type")};
            let trips_per_year: u32 =
                if let AnyValue::UInt32(int) = row.get(2).unwrap() {
                    *int
                } else { panic!("Wrong data type")};

            let from_cluster = cluster_adjacency.get_mut(&from_cluster_id);
            if let Some(from_cluster) = from_cluster {
                from_cluster.insert((to_cluster_id, trips_per_year));
            } else {
                let set = DashSet::with_capacity(1);
                set.insert((to_cluster_id, trips_per_year));
                cluster_adjacency.insert(from_cluster_id, set);
            }
        });

    drop(weighted_stops_adjacency);

    // Key is cluster-id, value are stop_ids
    let clusters: Map<u32, DashSet<u32>> = Map::with_capacity(stop_count as usize);
    // Put every stop into its own cluster at the beginning
    (0..stop_count).into_par_iter()
        .for_each(|index| {
            let stops_in_this_cluster = DashSet::with_capacity(1);
            stops_in_this_cluster.insert(index);
            clusters.insert(index, stops_in_this_cluster);
        });

    loop {
        let time = SystemTime::now();
        let best_pair = clusters.par_iter()
            // Get the best cluster to merge this with
            .filter_map(|entry| {
                let cluster_id = entry.key();
                let cluster_size = entry.value().len();
                let adjacent_clusters = cluster_adjacency.get(cluster_id);

                return if let Some(adjacent_clusters) = adjacent_clusters {
                    let best_adjacent_cluster = adjacent_clusters.iter()
                        .map(|adjacent_cluster| {
                            let (adjacent_cluster_id, trips_per_year) = adjacent_cluster.key();
                            //dbg!(&adjacent_cluster_id);
                            let adjacent_cluster_size = clusters.get(adjacent_cluster_id).unwrap()
                                .len();
                            let distance = evaluate_distance(cluster_size, adjacent_cluster_size, *trips_per_year);
                            (*adjacent_cluster_id, distance)
                        })
                        .min_by_key(|(_, distance)| {
                            OrderedFloat(*distance)
                        });

                    if let Some((best_to_cluster_with_id, distance)) = best_adjacent_cluster {
                        Some((*cluster_id, best_to_cluster_with_id, distance))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .min_by_key(|(_,_,distance)| OrderedFloat(*distance));

        if let Some((cluster_a_id, cluster_b_id, _)) = best_pair {
            let cluster_b_stops = clusters.remove(&cluster_b_id).unwrap().1;
            clusters.get_mut(&cluster_a_id).unwrap().extend(cluster_b_stops);

            // Update stop adjacency:
            // Remove item for cluster b
            let clusters_after_b = cluster_adjacency.remove(&cluster_b_id).unwrap().1;
            // Every cluster that was after b is now after cluster a, so add the trips to those
            let clusters_after_a = cluster_adjacency.get_mut(&cluster_a_id).unwrap();
            /*for (cluster_id, mut times_per_year_a) in &mut clusters_after_a {
                let after_b = clusters_after_b.iter()
                    .find(|after_b| after_b.0.clone() == cluster_id);
                if let Some(after_b) = after_b {
                    let (_, times_per_year_b) = *after_b;
                    times_per_year_a += times_per_year_b;
                }
            }*/
            // TODO: Further updates that are needed: Remove too large adjacencies (> MAX_CLUSTER_SIZE)

            println!("Merge took {:?}", time.elapsed().unwrap());
        } else {
            break;
        }
        break; // TODO: Remove this break to continue
    }

    Ok(())
}

#[inline]
fn evaluate_distance(size_u: usize, size_v: usize, weight_between_u_v: u32) -> f32 {
    let size_u = size_u as f32;
    let size_v = size_v as f32;
    let weight_between_u_v = weight_between_u_v as f32;
    (1.0 / size_u)
        * (1.0 / size_v)
        * (
            (weight_between_u_v / size_u.sqrt()) +
            (weight_between_u_v / size_v.sqrt())
        )
}