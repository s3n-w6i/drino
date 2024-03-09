use std::fmt;
use std::fmt::Display;
use polars::prelude::{LazyFrame, lit};
use crate::preprocessing_steps::step2_import_data::ImportStepExtra;
use crate::preprocessing_steps::step3_validate_data::ValidateStepOutput;

pub async fn merge(input: Vec<ValidateStepOutput>) -> Result<DatasetMergeOutput, MergeError> {
    // TODO: Actually merge datasets
    let first = input.into_iter()
        .filter(|data| !data.skip)
        .next()
        .expect("No valid dataset provided");
    let dataset_id = first.dataset.id;

    match first.extra.clone() { ImportStepExtra::Gtfs {
        calendar, stops, trips, stop_times, ..
    } => {
        let services = calendar
            .with_columns([
                lit(dataset_id.clone()).alias("dataset_id"),
            ]);
        let stops = stops
            .with_columns([
                lit(dataset_id.clone()).alias("dataset_id"),
            ]);
        let trips = trips.with_column(lit(dataset_id.clone()).alias("dataset_id"));
        let stop_times = stop_times.with_column(lit(dataset_id.clone()).alias("dataset_id"));

        Ok(DatasetMergeOutput {
            services, stops, trips, stop_times,
            import_extra: first.extra
        })
    } }
}

pub struct DatasetMergeOutput {
    pub services: LazyFrame, // corresponds to calendar.txt in GTFS
    pub stops: LazyFrame,
    pub trips: LazyFrame,
    pub stop_times: LazyFrame,
    pub import_extra: ImportStepExtra
}

#[derive(thiserror::Error, Debug)]
pub enum MergeError {
    Polars(#[from] polars::error::PolarsError),
}

impl Display for MergeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err: &dyn Display = match self {
            MergeError::Polars(err) => err,
        };
        write!(f, "{}", err)
    }
}