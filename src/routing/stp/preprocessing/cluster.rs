use polars::prelude::*;

// Implements merge-based clustering as described in section 3.1.2 of "Scalable Transfer Patterns"

const MAX_CLUSTER_SIZE: u32 = 1_500;

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
                    |acc, x| Ok(Some(acc + x)),  // ...then add all days in the week
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
        .agg([ sum("trips_per_year").alias("trips_per_year") ]);

    /*let mut stop_adj = weighted_stops_adjacency
        .clone()
        .join(
            stops.clone(),
            &[col("from_stop_id")],
            &[col("stop_id")],
            JoinArgs::new(JoinType::Left)
        )
        .select([
            col("lat").alias("from_lat"),
            col("lon").alias("from_lon"),
            col("to_stop_id"),
            col("trips_per_year")
        ])
        .join(
            stops.clone(),
            &[col("to_stop_id")],
            &[col("stop_id")],
            JoinArgs::new(JoinType::Left)
        )
        .select([
            col("lat").alias("to_lat"),
            col("lon").alias("to_lon"),
            col("from_lat"), col("from_lon"),
            col("trips_per_year")
        ])
        .collect()?;
    let mut file = std::fs::File::create("stop_adj.csv").unwrap();
    CsvWriter::new(&mut file).finish(&mut stop_adj).unwrap();*/

    // Put every stop into its own cluster at the beginning
    let mut clustered_stops = stops.clone()
        .select([col("stop_id")])
        .with_row_index("cluster_id", None)
        .collect()?;

    println!("{}", clustered_stops.clone());

    loop {
        let clusters = clustered_stops.clone().lazy()
            .group_by(["cluster_id"])
            .agg([len().alias("size"), col("stop_id").alias("stop_ids")])
            .collect()?;

        let adjacent_clusters = clustered_stops.clone().lazy()
            .join(
                weighted_stops_adjacency.clone().lazy(),
                [col("stop_id")],
                [col("from_stop_id")],
                JoinArgs::new(JoinType::Inner),
            )
            .select([
                col("cluster_id").alias("from_cluster_id"),
                col("to_stop_id"),
                col("trips_per_year")
            ])
            .join(
                clustered_stops.clone().lazy(),
                [col("to_stop_id")],
                [col("stop_id")],
                JoinArgs::new(JoinType::Inner),
            )
            .select([
                col("from_cluster_id"),
                col("cluster_id").alias("to_cluster_id"),
                col("trips_per_year")
            ])
            // Sum all trips per year across all connections of clusters
            .group_by([col("from_cluster_id"), col("to_cluster_id")])
            .agg([sum("trips_per_year").alias("trips_per_year")])
            // We cant merge clusters with our own cluster, so filter them out
            .filter(col("from_cluster_id").neq(col("to_cluster_id")));

        let adjacent_clusters_with_size = adjacent_clusters
            // First for the from-cluster
            .join(
                clusters.clone().lazy(),
                [col("from_cluster_id")],
                [col("cluster_id")],
                JoinArgs::new(JoinType::Left),
            )
            .select([
                col("from_cluster_id"), col("size").alias("from_cluster_size"),
                col("to_cluster_id"),
                col("trips_per_year")
            ])
            // Then for the to-cluster
            .join(
                clusters.clone().lazy(),
                [col("to_cluster_id")],
                [col("cluster_id")],
                JoinArgs::new(JoinType::Left),
            )
            .select([
                col("from_cluster_id"), col("from_cluster_size"),
                col("to_cluster_id"), col("size").alias("to_cluster_size"),
                col("trips_per_year")
            ]);

        // We only want to merge with those clusters, where the total number of stops doesn't exceed the
        // MAX_CLUSTER_SIZE
        let adjacent_clusters_with_small_enough_total_size = adjacent_clusters_with_size
            .filter(
                (col("from_cluster_size") + col("to_cluster_size"))
                    .lt_eq(lit(MAX_CLUSTER_SIZE))
            );
        let small_enough_count = adjacent_clusters_with_small_enough_total_size.clone()
            .first() // We only care for if there is none, so don't count all of them (1 or 0)
            .select([len().alias("count")])
            .collect()?.column("count")?.u32()?.get(0).unwrap();
        if small_enough_count < 1 {
            break;
        }

        let best_cluster_to_merge_with_by_each_cluster = adjacent_clusters_with_small_enough_total_size.clone()
            // Prepare columns needed for calculation of weight between two clusters
            // s(u): Size of Cluster u
            // s(v): Size of Cluster u
            // w(u,v): TODO
            .select([
                col("from_cluster_id"), col("to_cluster_id"),
                col("from_cluster_size").alias("s(u)"),
                col("to_cluster_size").alias("s(v)"),
                col("trips_per_year").alias("w(u,v)"),
            ])
            // Calculate the weight between adjacent clusters
            .with_column(
                weight_function.clone()
                    .cast(DataType::Float32) // Reduce space consumption
                    .alias("weight")
            )
            .group_by([col("from_cluster_id")])
            .agg([
                col("to_cluster_id").alias("to_cluster_ids"),
                col("weight")
            ])
            .with_column(
                col("weight").list()
                    .arg_max()
                    .alias("best_to_cluster_index")
            )
            // Use the calculated index to retrieve the id of the best cluster
            .select([
                col("from_cluster_id"),
                col("to_cluster_ids").list()
                    .get(col("best_to_cluster_index"))
                    .alias("best_to_cluster_id"),
                col("weight").list()
                    .get(col("best_to_cluster_index"))
                    .alias("best_weight")
            ]);

        let best_cluster_pair = best_cluster_to_merge_with_by_each_cluster
            .sort("best_weight", SortOptions::default())
            .last()
            .collect()?;

        println!("{}", best_cluster_pair);

        // Merge clusters by making every stop in the best_to_cluster have the same cluster id as the
        // from cluster
        clustered_stops = clustered_stops.clone().lazy()
            .join(
                best_cluster_pair.lazy(),
                [col("cluster_id")],
                [col("best_to_cluster_id")],
                JoinArgs::new(JoinType::Left),
            )
            .select([
                col("stop_id"),
                // Take the new cluster id first if not null, otherwise keep using the old cluster id
                coalesce(&[col("from_cluster_id"), col("cluster_id")])
                    .alias("cluster_id")
            ])
            .collect()?;


    }

    drop(weighted_stops_adjacency);

    let clusters = clustered_stops.lazy()
        .group_by(["cluster_id"])
        .agg([len().alias("size"), col("stop_id").alias("stop_ids")]);
    println!("{}", clusters.clone().top_k(50, [col("size")], [false], false, false).clone().collect()?);

    let mut file = std::fs::File::create("clusters.csv").unwrap();
    CsvWriter::new(&mut file).finish(&mut clusters.clone().explode(&[col("stop_ids")]).collect()?).unwrap();


    Ok(())
}