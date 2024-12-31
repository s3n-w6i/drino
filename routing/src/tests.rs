#[cfg(test)]
pub(crate) mod case_1 {
    use crate::algorithm::PreprocessingInput;
    use chrono::{NaiveDate, TimeDelta};
    use polars::df;
    use polars::error::PolarsResult;
    use polars::prelude::IntoLazy;

    pub(crate) fn generate_preprocessing_input() -> PolarsResult<PreprocessingInput> {
        Ok(PreprocessingInput {
            services: df![
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
        ]?.lazy(),
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
            "arrival_time" => [TimeDelta::seconds(100), TimeDelta::seconds(500)],
            "departure_time" => [TimeDelta::seconds(100), TimeDelta::seconds(500)],
            "stop_sequence" => [0u32, 1],
        ]?.lazy(),
        })
    }
}

