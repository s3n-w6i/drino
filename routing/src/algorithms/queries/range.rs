use crate::algorithms::errors::{MultiQueryResult, QueryResult};
use crate::algorithms::queries::cardinality::{All, Multiple, Single, TargetCardinality};
use crate::algorithms::queries::QueryType;
use crate::journey::Journey;
use chrono::{DateTime, TimeDelta, Utc};
use common::types::StopId;
use hashbrown::HashSet;
use serde::Deserialize;
use serde_with::serde_derive::Serialize;
use serde_with::serde_as;

/// A range query asks for all optimal journeys between stations in a specified time range

pub struct Range {}
impl QueryType for Range {
    type Input = RangeInput;
}

#[serde_as]
#[derive(Deserialize)]
pub struct RangeInput {
    pub(crate) earliest_departure: DateTime<Utc>,
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    pub(crate) range: TimeDelta,
    pub(crate) start: StopId,
}

impl RangeInput {
    fn from_absolute(earliest: DateTime<Utc>, latest: DateTime<Utc>, start: StopId) -> Self {
        Self {
            earliest_departure: earliest,
            range: latest - earliest,
            start,
        }
    }
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub struct RangeOutput {
    pub(crate) journeys: HashSet<Journey>,
}

impl TargetCardinality<Range> for Single {
    type Output = RangeOutput;
}
impl<'a> TargetCardinality<Range> for Multiple<'a> {
    type Output = Vec<RangeOutput>;
}
impl TargetCardinality<Range> for All {
    type Output = RangeOutput;
}
