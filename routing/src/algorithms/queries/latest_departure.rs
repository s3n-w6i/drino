use crate::algorithms::queries::cardinality::{Single, TargetCardinality};
use crate::algorithms::queries::QueryType;
use crate::journey::Journey;
use chrono::{DateTime, Utc};
use common::types::StopId;
use serde::Deserialize;
use serde_with::serde_derive::Serialize;

/// The latest departure query asks for the one optimal journey when needing to arrive at or before
/// a specified point in time

pub struct LatestDeparture {}
impl QueryType for LatestDeparture {
    type Input = LatestDepartureInput;
}

#[derive(Deserialize)]
pub struct LatestDepartureInput {
    pub(crate) latest_arrival: DateTime<Utc>,
    pub(crate) start: StopId,
}

#[derive(Serialize)]
pub struct LatestDepartureOutput {
    pub(crate) journey: Journey,
}

impl TargetCardinality<LatestDeparture> for Single {
    type Output = LatestDepartureOutput;
}
