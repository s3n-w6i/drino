use std::fmt;
use std::fmt::{Debug, Display};

use chrono::{DateTime, Duration, TimeDelta, Utc};
use hashbrown::HashSet;
use indicatif::MultiProgress;
use polars::prelude::LazyFrame;

use crate::transfers::TransferError;
use common::types::{StopId, TripId};

pub trait RoutingAlgorithm {}

pub trait PreprocessInit: RoutingAlgorithm + Sized {
    fn preprocess(input: PreprocessingInput, progress_bars: Option<&MultiProgress>) -> PreprocessingResult<Self>;
}


pub trait FromDiskInit: RoutingAlgorithm + Sized {
    fn load_from_disk() -> PreprocessingResult<Self>;
    fn save_to_disk() -> ();
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


trait QueryType: Sized {}

// The earliest arrival query asks for the one optimal journey when departing at or after a
// specified point in time
pub struct EarliestArrivalQuery {}

// A range query asks for all optimal journeys between stations in a specified time range
pub struct RangeQuery {}

impl QueryType for EarliestArrivalQuery {}

impl QueryType for RangeQuery {}

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

trait QueryInput<T: QueryType>: Sized {}

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

impl QueryInput<EarliestArrivalQuery> for EarliestArrival {}

impl QueryInput<RangeQuery> for Range {}

trait QueryOutput<T: QueryType>: Sized + Debug {}

#[derive(Debug)]
pub struct EarliestArrivalOutput {
    pub(crate) journey: Journey,
}

#[derive(Debug)]
pub struct RangeOutput {
    pub(crate) journeys: HashSet<Journey>,
}

impl QueryOutput<EarliestArrivalQuery> for EarliestArrivalOutput {}

impl QueryOutput<RangeQuery> for RangeOutput {}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Leg {
    Ride { trip: TripId, boarding_stop: StopId, alight_stop: StopId, boarding_time: DateTime<Utc>, alight_time: DateTime<Utc> },
    Transfer { start: StopId, end: StopId, duration: Duration },
}

impl Leg {
    pub(crate) fn start(&self) -> &StopId {
        match self {
            Leg::Ride { boarding_stop: start, .. } | Leg::Transfer { start, .. } => start,
        }
    }
    
    pub(crate) fn end(&self) -> &StopId {
        match self {
            Leg::Ride { alight_stop: end, .. } | Leg::Transfer { end, .. } => end,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Journey {
    pub(crate) legs: Vec<Leg>,
}

impl Journey {
    // Return the time at which this journey will start
    // This is done by summing up all transfer durations before the first fixed departure (aka a
    // ride). The transfer durations will then be subtracted from that first departure date-time.
    // If the Journey only consists of transfers, then None will be returned.
    pub(crate) fn departure(&self) -> Option<DateTime<Utc>> {
        let first_ride = self.legs.iter().find(|leg| matches!(leg, Leg::Ride { .. }));

        if let Some(first_ride) = first_ride {
            let start_transfers_duration: TimeDelta = self.legs.iter()
                .take_while(|leg| matches!(leg, Leg::Transfer {..}))
                .map(|leg| {
                    match leg {
                        Leg::Transfer { duration, .. } => duration,
                        _ => panic!("A ride leg cannot occur here, since we only take while legs are transfers!")
                    }
                })
                .sum();
            if let Leg::Ride { boarding_time, .. } = first_ride {
                Some(*boarding_time - start_transfers_duration)
            } else {
                panic!("The first_ride leg cannot not be a ride!");
            }
        } else {
            None
        }
    }

    // Return the time at which this journey will end at the destination
    // This is done by summing up all transfer durations from back to front, until we hit a ride.
    // The transfer durations will then be added to the arrival date-time of the last ride.
    // If the Journey only consists of transfers, then None will be returned.
    pub(crate) fn arrival(&self) -> Option<DateTime<Utc>> {
        let legs_reversed = self.legs.iter().rev();

        let last_ride = legs_reversed.clone().find(|leg| matches!(leg, Leg::Ride { .. }));

        if let Some(last_ride) = last_ride {
            let end_transfers_duration: TimeDelta = legs_reversed.clone()
                .take_while(|leg| matches!(leg, Leg::Transfer {..}))
                .map(|leg| {
                    match leg {
                        Leg::Transfer { duration, .. } => duration,
                        _ => panic!("A ride leg cannot occur here, since we only take while legs are transfers!")
                    }
                })
                .sum();
            if let Leg::Ride { alight_time, .. } = last_ride {
                Some(*alight_time + end_transfers_duration)
            } else {
                panic!("The last_ride leg cannot not be a ride!");
            }
        } else {
            None
        }
    }

    pub(crate) fn arrival_when_starting_at(&self, departure: DateTime<Utc>) -> Option<DateTime<Utc>> {
        if let Some(journey_departure) = self.departure() {
            // This journey has a departure date-time, that we cannot miss. If we do, we will not arrive
            if journey_departure < departure {
                None
            } else {
                self.arrival()
            }
        } else {
            // This journey does not have a fixed departure date-time, so calculate the arrival based
            // on the duration.
            // Example: Only walking from A to B. This can be done at any time.
            let duration: TimeDelta = self.legs.iter()
                .map(|leg| {
                    match leg {
                        Leg::Transfer { duration, .. } => duration,
                        _ => panic!("Journey was not expected to have a ride leg, since its departure is None")
                    }
                })
                .sum();
            Some(departure + duration)
        }
    }

    pub(crate) fn start(&self) -> &StopId {
        self.legs.first().unwrap().start()
    }

    pub(crate) fn end(&self) -> &StopId {
        self.legs.last().unwrap().end()
    }
}

impl From<Vec<Leg>> for Journey {
    fn from(legs: Vec<Leg>) -> Self {
        Self { legs }
    }
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