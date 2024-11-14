use chrono::{DateTime, Duration, Utc};
use hashbrown::{HashMap, HashSet};

use crate::algorithm::{QueryResult, RoutingAlgorithm};
use crate::journey::Journey;
use crate::transfers::TransferProvider;
use common::types::{LineId, SeqNum, StopId, TripId};

mod state;
mod preprocessing;
mod routing;

pub type TripAtStopTimeMap = HashMap<(TripId, StopId), DateTime<Utc>>;
pub type TripsByLineAndStopMap = HashMap<(LineId, StopId), Vec<(DateTime<Utc>, TripId)>>;

pub struct RaptorAlgorithm {
    pub(crate) stops: Vec<StopId>,

    pub(crate) stops_by_line: HashMap<LineId, Vec<StopId>>,
    pub(crate) lines_by_stops: HashMap<StopId, HashSet<(LineId, SeqNum)>>,

    // <(trip_id, stop_id), departure_time>
    pub(crate) departures: TripAtStopTimeMap,
    // <(trip_id, stop_id), arrival_time>
    pub(crate) arrivals: TripAtStopTimeMap,

    // Vec has to be sorted from earliest to latest
    // DateTime is departure
    pub(crate) trips_by_line_and_stop: TripsByLineAndStopMap,

    pub(crate) transfer_provider: Box<dyn TransferProvider + Send + Sync>,
}

impl RoutingAlgorithm for RaptorAlgorithm {}