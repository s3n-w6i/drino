use serde_with::DisplayFromStr;
use crate::algorithms::queries::cardinality::{All, Multiple, Single, TargetCardinality};
use crate::algorithms::queries::QueryType;
use crate::journey::Journey;
use chrono::{DateTime, TimeDelta, Utc};
use common::types::StopId;
use hashbrown::HashSet;
use serde::Deserialize;
use serde_with::formats::Flexible;
use serde_with::serde_as;
use serde_with::serde_derive::Serialize;

/// A range query asks for all optimal journeys between stations in a specified time range

pub struct Range {}
impl QueryType for Range {
    type Input = RangeInput;
}

#[serde_as]
#[derive(Deserialize)]
pub struct RangeInput {
    #[serde_as(as = "serde_with::TimestampSeconds<String, Flexible>")]
    pub(crate) earliest_departure: DateTime<Utc>,
    #[serde_as(as = "serde_with::DurationSeconds<String>")]
    pub(crate) range: TimeDelta,
    #[serde_as(as = "DisplayFromStr")]
    pub(crate) start: StopId,
}

impl RangeInput {
    pub fn from_absolute(earliest: DateTime<Utc>, latest: DateTime<Utc>, start: StopId) -> Self {
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
