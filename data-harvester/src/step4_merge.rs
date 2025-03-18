use crate::step2_import::ImportStepExtra;
use crate::step3_validate::ValidateStepOutput;
use polars::prelude::{lit, LazyFrame};
use std::fmt;
use std::fmt::Display;

pub async fn merge<'a>(input: Vec<ValidateStepOutput<'a>>) -> Result<DatasetMergeOutput, MergeError> {
    // TODO: Actually merge datasets
    let first = input.into_iter()
        .filter(|data| !data.skip)
        .next()
        .ok_or(MergeError::NoDatasets())?;
    let dataset_id = &first.dataset.id;

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
    NoDatasets(),
}

impl Display for MergeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err = match self {
            MergeError::Polars(err) => err.fmt(f),
            MergeError::NoDatasets { .. } => write!(f, "No datasets were provided"),
        };
        err
    }
}