use chrono::{DateTime, Duration, Utc};
use hashbrown::{HashMap, HashSet};

use crate::algorithm::{Journey, QueryResult, RoutingAlgorithm};
use crate::transfers::CrowFlyTransferProvider;
use common::types::{LineId, SeqNum, StopId, TripId};

mod state;
mod preprocessing;
mod routing;

#[derive(Clone)]
pub struct RaptorAlgorithm {
    pub stops: Vec<StopId>,
    stops_by_line: HashMap<LineId, Vec<StopId>>,
    lines_by_stops: HashMap<StopId, HashSet<(LineId, SeqNum)>>,
    // <(trip_id, stop_id), departure_time>
    departures: HashMap<(TripId, StopId), DateTime<Utc>>,
    // <(trip_id, stop_id), arrival_time>
    arrivals: HashMap<(TripId, StopId), DateTime<Utc>>,
    // Vec has to be sorted from earliest to latest
    // DateTime is departure
    trips_by_line_and_stop: HashMap<(LineId, StopId), Vec<(DateTime<Utc>, TripId)>>,
    transfer_provider: CrowFlyTransferProvider,
}

impl RoutingAlgorithm for RaptorAlgorithm {}