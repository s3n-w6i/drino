use std::fmt;
use std::fmt::{Debug, Display};
use chrono::{DateTime, TimeDelta, Utc};
use hashbrown::HashSet;
use indicatif::MultiProgress;
use polars::prelude::LazyFrame;

use crate::transfers::TransferError;
use common::types::StopId;
use crate::journey::Journey;

pub trait RoutingAlgorithm {}

pub trait PreprocessInit: RoutingAlgorithm + Sized {
    fn preprocess(input: PreprocessingInput, progress_bars: Option<&MultiProgress>) -> PreprocessingResult<Self>;
}


pub trait FromDiskInit: RoutingAlgorithm + Sized {
    fn load_from_disk() -> PreprocessingResult<Self>;
    fn save_to_disk();
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
}

impl Display for PreprocessingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err: &dyn Display = match self {
            PreprocessingError::Polars(err) => err,
            PreprocessingError::KMeans(err) => err,
        };
        write!(f, "{}", err)
    }
}


// The earliest arrival query asks for the one optimal journey when departing at or after a
// specified point in time
pub struct EarliestArrivalQuery {}

// A range query asks for all optimal journeys between stations in a specified time range
pub struct RangeQuery {}

pub trait QueryTargetCardinality: Sized + Clone {}

#[derive(Clone)]
pub struct Single {
    pub(crate) target: StopId,
}

#[derive(Clone)]
pub struct Multiple<'lifetime> {
    pub(crate) targets: &'lifetime Vec<StopId>,
}

#[derive(Clone)]
pub struct All {}

impl QueryTargetCardinality for Single {}

impl QueryTargetCardinality for Multiple<'static> {}

impl QueryTargetCardinality for All {}

pub struct EarliestArrival {
    pub(crate) departure: DateTime<Utc>,
    pub(crate) start: StopId,
}

pub struct Range {
    pub(crate) earliest_departure: DateTime<Utc>,
    pub(crate) range: TimeDelta,
    pub(crate) start: StopId,
}

impl Range {
    fn from_absolute(earliest: DateTime<Utc>, latest: DateTime<Utc>, start: StopId) -> Self {
        Self {
            earliest_departure: earliest,
            range: latest - earliest,
            start,
        }
    }
}


#[derive(Debug)]
pub struct EarliestArrivalOutput {
    pub(crate) journey: Journey,
}

#[derive(Debug)]
pub struct RangeOutput {
    pub(crate) journeys: HashSet<Journey>,
}


pub trait SingleEarliestArrival: RoutingAlgorithm {
    fn query_ea(&self, input: EarliestArrival, cardinality: Single) -> QueryResult<EarliestArrivalOutput>;
}

pub trait SingleRange: RoutingAlgorithm {
    fn query_range(&self, input: Range, cardinality: Single) -> QueryResult<RangeOutput>;
}

pub trait MultiEarliestArrival: RoutingAlgorithm {
    fn query_ea_multi(&self, input: EarliestArrival, cardinality: Multiple) -> MultiQueryResult<EarliestArrivalOutput>;
}

pub trait MultiRange: RoutingAlgorithm {
    fn query_range_multi(&self, input: Range, cardinality: Multiple) -> QueryResult<RangeOutput>;
}

pub trait AllEarliestArrival: RoutingAlgorithm {
    fn query_ea_all(&self, input: EarliestArrival) -> MultiQueryResult<EarliestArrivalOutput>;
}

pub trait AllRange: RoutingAlgorithm {
    fn query_range_all(&self, input: Range) -> QueryResult<RangeOutput>;
}


pub type QueryResult<O> =
Result<O, QueryError>;
pub type MultiQueryResult<O> =
Result<Vec<O>, QueryError>;

#[derive(thiserror::Error, Debug)]
pub enum QueryError {
    Polars(#[from] polars::error::PolarsError),
    NoRouteFound,
    TransferError(#[from] TransferError),
}

impl Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err: &dyn Display = match self {
            QueryError::Polars(err) => err,
            QueryError::NoRouteFound => &"No route found",
            QueryError::TransferError(err) => err
        };
        write!(f, "{}", err)
    }
}

impl From<Journey> for EarliestArrivalOutput {
    fn from(journey: Journey) -> Self {
        Self { journey }
    }
}