use crate::algorithms::errors::{MultiQueryResult, QueryResult};
use crate::algorithms::queries::cardinality::{All, Multiple, Single, TargetCardinality};
use crate::algorithms::queries::QueryType;
use crate::journey::Journey;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use common::types::StopId;
use serde_with::serde_derive::Serialize;

/// The earliest arrival query asks for the one optimal journey when departing at or after a
/// specified point in time

pub struct EarliestArrival {}
impl QueryType for EarliestArrival {
    type Input = EarliestArrivalInput;
}

#[derive(Deserialize)]
pub struct EarliestArrivalInput {
    pub(crate) earliest_departure: DateTime<Utc>,
    pub(crate) start: StopId,
}

#[derive(Serialize, Debug, Eq, PartialEq)]
pub struct EarliestArrivalOutput {
    pub journey: Journey,
}

// Allow all target cardinalities for earliest arrival
impl TargetCardinality<EarliestArrival> for Single {
    type Output = EarliestArrivalOutput;
}
impl<'a> TargetCardinality<EarliestArrival> for Multiple<'a> {
    type Output = Vec<EarliestArrivalOutput>;
}
impl TargetCardinality<EarliestArrival> for All {
    type Output = Vec<EarliestArrivalOutput>;
}
