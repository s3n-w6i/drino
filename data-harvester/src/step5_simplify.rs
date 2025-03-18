use crate::step4_merge::DatasetMergeOutput;
use common::util::df::{write_df_to_file, FileType};
use polars::frame::DataFrame;
use polars::prelude::{col, Column, IntoLazy, JoinArgs, JoinType};
use polars::series::Series;
use std::fmt;
use std::fmt::Display;
use routing::algorithms::initialization::PreprocessingInput;

fn assign_new_ids(
    mut frame: DataFrame,
    name: &str
) -> Result<DataFrame, SimplifyError> {
    let num = frame.get_columns().first().unwrap().len() as u32;

    let mut new_ids = Column::from(Series::from_iter(0..num));
    new_ids.rename(name.into());

    frame.with_column(new_ids)?;

    Ok(frame)
}

pub async fn simplify(
    DatasetMergeOutput {
        stops,
        trips,
        services,
        stop_times,
        ..
    }: DatasetMergeOutput
) -> Result<PreprocessingInput, SimplifyError> {
    // Turn stop ids into integers
    let stops = stops
        // Only include stops that are used in trips
        .semi_join(
            stop_times.clone(),
            col("stop_id"),
            col("stop_id")
        ).select([
            // Keep "old" id-pairs (stop_id + dataset_id) so that we can match in other tables
            col("stop_id").alias("stop_id_in_dataset"),
            col("dataset_id"),
            col("stop_lat").alias("lat"),
            col("stop_lon").alias("lon"),
        ]);

    // Generate a new stop_id
    let stops = assign_new_ids(stops.collect()?, "stop_id")?;

    write_df_to_file("data/tmp/simplify/stops.parquet".into(), FileType::PARQUET, stops.clone())?;
    let stops = stops.lazy();

    let trips = trips
        .select([
            col("trip_id").alias("trip_id_in_dataset"),
            col("route_id").alias("route_id_in_dataset"),
            col("service_id").alias("service_id_in_dataset"),
            col("dataset_id"),
        ]);

    let trips = assign_new_ids(trips.collect()?, "trip_id")?;

    write_df_to_file("data/tmp/simplify/trips.parquet".into(), FileType::PARQUET, trips.clone())?;
    let trips = trips.lazy();

    let services = services
        .select([
            col("dataset_id"),
            col("service_id").alias("service_id_in_dataset"),
            col("monday"), col("tuesday"), col("wednesday"),
            col("thursday"), col("friday"), col("saturday"),
            col("sunday"), col("start_date"), col("end_date")
        ]);

    let services = assign_new_ids(services.collect()?, "service_id")?;

    write_df_to_file("data/tmp/simplify/services.parquet".into(), FileType::PARQUET, services.clone())?;
    let services = services.lazy();

    let stop_times = stop_times
        .select([
            col("trip_id").alias("trip_id_in_dataset"),
            col("arrival_time"),
            col("departure_time"),
            col("stop_id").alias("stop_id_in_dataset"),
            col("dataset_id"),
            col("stop_sequence"),
        ])
        // Convert stop_ids to numeric ones
        .join(
            stops.clone().select([col("dataset_id"), col("stop_id_in_dataset"), col("stop_id")]),
            [col("dataset_id"), col("stop_id_in_dataset")],
            [col("dataset_id"), col("stop_id_in_dataset")],
            JoinArgs::new(JoinType::Inner),
        )
        // Convert trip_ids to numeric ones
        .join(
            trips.clone().select([col("dataset_id"), col("trip_id_in_dataset"), col("trip_id")]),
            [col("dataset_id"), col("trip_id_in_dataset")],
            [col("dataset_id"), col("trip_id_in_dataset")],
            JoinArgs::new(JoinType::Inner),
        );

    write_df_to_file("data/tmp/simplify/stop_times.parquet".into(), FileType::PARQUET, stop_times.clone().collect()?)?;

    let stop_times = stop_times.drop(["stop_id_in_dataset"])
        .drop(["dataset_id", "trip_id_in_dataset"]);

    let trips = trips
        .join(
            services.clone().select([col("dataset_id"), col("service_id_in_dataset"), col("service_id")]),
            [col("dataset_id"), col("service_id_in_dataset")],
            [col("dataset_id"), col("service_id_in_dataset")],
            JoinArgs::new(JoinType::Inner)
        )
        .drop(["service_id_in_dataset"]);

    let stops = stops
        .drop([
            "stop_id_in_dataset", "dataset_id"
        ]);

    let trips = trips.drop([
        "trip_id_in_dataset", "dataset_id"
    ]);

    let services = services.drop([
        "service_id_in_dataset", "dataset_id"
    ]);

    Ok(PreprocessingInput {
        services,
        stops,
        trips,
        stop_times,
    })
}

#[derive(thiserror::Error, Debug)]
pub enum SimplifyError {
    Polars(#[from] polars::error::PolarsError),
}

impl Display for SimplifyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err: &dyn Display = match self {
            SimplifyError::Polars(err) => err,
        };
        write!(f, "{}", err)
    }
}