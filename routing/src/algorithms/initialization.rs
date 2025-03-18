use crate::algorithms::RoutingAlgorithm;
use polars::prelude::LazyFrame;
use std::fmt;
use std::fmt::Display;

pub trait ByPreprocessing: RoutingAlgorithm {
    fn preprocess(input: PreprocessingInput, save_to_disk: bool) -> PreprocessingResult<Self>;
}

pub trait FromDisk: RoutingAlgorithm {
    fn from_disk() -> PreprocessingResult<Self>;
}

#[derive(Clone)]
pub struct PreprocessingInput {
    // corresponds to calendar.txt in GTFS
    pub services: LazyFrame,
    pub stops: LazyFrame,
    pub trips: LazyFrame,
    pub stop_times: LazyFrame,
}

pub type PreprocessingResult<T> = Result<T, PreprocessingError>;

#[derive(thiserror::Error, Debug)]
pub enum PreprocessingError {
    Polars(#[from] polars::error::PolarsError),
    KMeans(#[from] linfa_clustering::KMeansError),
    IO(#[from] std::io::Error),
    GeoArrow(#[from] geoarrow::error::GeoArrowError),
    Arrow(#[from] arrow_schema::ArrowError),
    BuildLines(#[from] common::util::geoarrow_lines::Error),
}

impl Display for PreprocessingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err: &dyn Display = match self {
            PreprocessingError::Polars(err) => err,
            PreprocessingError::KMeans(err) => err,
            PreprocessingError::IO(err) => err,
            PreprocessingError::GeoArrow(err) => err,
            PreprocessingError::Arrow(err) => err,
            PreprocessingError::BuildLines(err) => err,
        };
        write!(f, "{}", err)
    }
}
