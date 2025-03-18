use crate::algorithms::initialization::PreprocessingInput;
use chrono::NaiveDate;
use polars::datatypes::{AnyValue, TimeUnit};
use polars::df;
use polars::error::PolarsResult;
use polars::prelude::{IntoLazy, LazyFrame};

fn single_all_week_service() -> PolarsResult<LazyFrame> {
    Ok(df![
        "service_id" => [0u32],
        "monday" => [true],
        "tuesday" => [true],
        "wednesday" => [true],
        "thursday" => [true],
        "friday" => [true],
        "saturday" => [true],
        "sunday" => [true],
        "start_date" => [NaiveDate::from_ymd_opt(1970, 1, 1)],
        "end_date" => [NaiveDate::from_ymd_opt(2070, 1, 1)],
    ]?.lazy())
}

/// Test case 1 is probably the most simple case (that would still make sense):
/// - 2 stops
/// - 1 trip connecting the two stops
/// One can only ride from stop 0 to stop 1. They are *very* far from one another, so walking is not
/// really an option.
pub(crate) mod case_1 {
    use super::*;

    pub(crate) fn generate_preprocessing_input() -> PolarsResult<PreprocessingInput> {
        Ok(PreprocessingInput {
            services: single_all_week_service()?,
            stops: df![
                "stop_id" => [0u32, 1],
                "lat" => [0f32, 45.0],
                "lon" => [0f32, 45.0],
            ]?.lazy(),
            trips: df![
                "trip_id" => [0u32],
                "service_id" => [0u32],
            ]?.lazy(),
            stop_times: df![
                "trip_id" => [0u32, 0],
                "stop_id" => [0u32, 1],
                "arrival_time" => [duration(100), duration(500)],
                "departure_time" => [duration(100), duration(500)],
                "stop_sequence" => [0u32, 1],
            ]?.lazy(),
        })
    }
}

/// Test case 2 has
/// - 3 stops
/// - 2 trips
/// The longest journey one can take is: 0 ---Ride--> 1 ---Ride--> 2. Going backwards is not a thing
/// here. Walking is also not an option, as stops are very far from another.
pub(crate) mod case_2 {
    use super::*;

    pub(crate) fn generate_preprocessing_input() -> PolarsResult<PreprocessingInput> {
        Ok(PreprocessingInput {
            services: single_all_week_service()?,
            stops: df![
                "stop_id" => [0u32, 1, 2],
                "lat" => [0f32, 45.0, -45.0],
                "lon" => [0f32, 45.0, -45.0],
            ]?.lazy(),
            trips: df![
                "trip_id" => [0u32, 1],
                "service_id" => [0u32, 0]
            ]?.lazy(),
            stop_times: df![
                "trip_id" => [0u32, 0, 1, 1],
                "stop_id" => [0u32, 1, 1, 2],
                "arrival_time" => [duration(100), duration(500), duration(1_000), duration(1_500)],
                "departure_time" => [duration(100), duration(500), duration(1_000), duration(1_500)],
                "stop_sequence" => [0u32, 1, 0, 1],
            ]?.lazy(),
        })
    }
}

/// Test case 3 has
/// - 4 stops
/// - 2 trips (one from stop 0 to 1, one from 2 to 3)
/// - 1 feasible walking connection (between stops 1 and 2)
/// The longest journey one can take is: 0 ---Ride--> 1 ---Transfer--> 2 ---Ride--> 3. Going
/// backwards is not a thing.
pub(crate) mod case_3 {
    use super::*;
    
    pub(crate) fn generate_preprocessing_input() -> PolarsResult<PreprocessingInput> {
        Ok(PreprocessingInput {
            services: single_all_week_service()?,
            stops: df![
                "stop_id" => [0u32, 1, 2, 3],
                "lat" => [0f32, 45.0, 45.01, 45.0],
                "lon" => [0f32, 45.0, 45.01, -45.0],
            ]?.lazy(),
            trips: df![
                "trip_id" => [0u32, 1],
                "service_id" => [0u32, 0],
            ]?.lazy(),
            stop_times: df![
                "trip_id" => [0u32, 0, 1, 1],
                "stop_id" => [0u32, 1, 2, 3],
                "arrival_time" => [duration(100), duration(500), duration(1_000), duration(1_500)],
                "departure_time" => [duration(100), duration(500), duration(1_000), duration(1_500)],
                "stop_sequence" => [0u32, 1, 0, 1],
            ]?.lazy(),
        })
    }
}

/// Helper function for generating arrival and departure times more concisely
fn duration<'a>(seconds: i64) -> AnyValue<'a> {
    AnyValue::Duration(seconds * 1_000, TimeUnit::Milliseconds)
}