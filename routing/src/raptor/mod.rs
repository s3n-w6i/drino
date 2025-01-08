use crate::algorithm::{QueryResult, RoutingAlgorithm};
use crate::journey::Journey;
use crate::transfers::TransferProvider;
use chrono::{DateTime, Duration, Utc};
use common::types::{LineId, SeqNum, StopId, TripId};
use hashbrown::{HashMap, HashSet};

mod preprocessing;
mod routing;
mod state;
mod tests;

type GlobalStopId = StopId;
type LocalStopId = StopId;

/// <(trip_id, stop_id, visit_idx), time>
/// the visit_idx is there, since a trip could visit the same stop multiple times (think round trips)
pub type TripAtStopTimeMap = HashMap<(TripId, LocalStopId, u32), DateTime<Utc>>;
pub type TripsByLineAndStopMap =
    HashMap<(LineId, LocalStopId), Vec<(DateTime<Utc>, TripId)>>;

pub type StopsByLineMap = HashMap<LineId, Vec<(LocalStopId, u32)>>;
pub type LinesByStopMap = HashMap<LocalStopId, HashSet<(LineId, SeqNum)>>;

pub struct RaptorAlgorithm {
    pub(crate) stop_mapping: StopMapping,

    /// <line_id, [stop_id, visit_idx]>
    pub(crate) stops_by_line: StopsByLineMap,
    pub(crate) lines_by_stops: LinesByStopMap,

    // <(trip_id, stop_id, visit_idx), departure_time>
    pub(crate) departures: TripAtStopTimeMap,
    // <(trip_id, stop_id, visit_idx), arrival_time>
    pub(crate) arrivals: TripAtStopTimeMap,

    // Vec has to be sorted from earliest to latest
    // DateTime is departure
    pub(crate) trips_by_line_and_stop: TripsByLineAndStopMap,

    pub(crate) transfer_provider: Box<dyn TransferProvider + Send + Sync>,
}

impl RoutingAlgorithm for RaptorAlgorithm {}

impl RaptorAlgorithm {
    fn local_stop_ids(&self) -> impl Iterator<Item = LocalStopId> {
        (0..self.num_stops()).map(|x| StopId(x as u32))
    }

    pub(crate) fn num_stops(&self) -> usize {
        // Since each stop has also got a global ID, use the number of those IDs to determine how many
        // stops there are.
        self.stop_mapping.0.len()
    }
}

/// In order to simplify lookup of data, the passed stop IDs will be transformed to local stop
/// ids that start at zero and assign a new stop ID to each stop continuously. The index of
/// stop_mapping is the local stop ID, the value at that index is the global stop ID.
#[derive(Debug)]
pub(crate) struct StopMapping(pub(crate) Vec<GlobalStopId>);

impl StopMapping {
    /// Translates a local stop ID into a global stop ID
    fn translate_to_global(&self, local_stop_id: LocalStopId) -> GlobalStopId {
        self.0[local_stop_id.0 as usize]
    }

    /// Translates a global stop ID into a local stop ID
    // TODO: Return the result, maybe use a separate hash map to speed up the lookup?
    fn translate_to_local(&self, global_stop_id: GlobalStopId) -> LocalStopId {
        let idx = self.0.iter().position(|stop_id| stop_id == &global_stop_id);

        StopId(idx.unwrap() as u32)
    }
}
