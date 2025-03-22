use crate::algorithms::RoutingAlgorithm;
use crate::journey::Journey;
use crate::transfers::TransferProvider;
use chrono::{DateTime, Duration, Utc};
use common::types::trip::{AnyTripId, OneOff, Recurring, TripType};
use common::types::{LineId, SeqNum, StopId};
use hashbrown::{HashMap, HashSet};

mod preprocessing;
mod routing;
mod state;
#[cfg(test)]
mod tests;

type GlobalStopId = StopId;
type LocalStopId = StopId;

pub type TripsByLineAndStopMap<TT: TripType> =
    HashMap<(LineId, LocalStopId), Vec<(DateTime<Utc>, TT::Id)>>;

pub type StopsByLineMap = HashMap<LineId, Vec<(LocalStopId, u32)>>;
pub type LinesByStopMap = HashMap<LocalStopId, HashSet<(LineId, SeqNum)>>;

pub struct RaptorAlgorithm {
    pub(crate) stop_mapping: StopMapping,

    // STOPS AND LINES
    // <line_id, [stop_id, visit_idx]>
    pub(crate) stops_by_line: StopsByLineMap,
    pub(crate) lines_by_stops: LinesByStopMap,

    // ARRIVALS & DEPARTURES
    pub(crate) arrivals: AnyTripAtStopTime,
    pub(crate) departures: AnyTripAtStopTime,

    // TRIPS
    // Vec has to be sorted from earliest to latest
    // DateTime is departure at the stop
    pub(crate) one_off_trips_by_line_and_stop: TripsByLineAndStopMap<OneOff>,
    pub(crate) recurring_trips_by_line_and_stop: TripsByLineAndStopMap<Recurring>,

    // TRANSFERS
    pub(crate) transfer_provider: Box<dyn TransferProvider + Send + Sync>,
}

/// <(trip_id, stop_id, visit_idx), time>
/// - visit_idx is there, since a trip could visit the same stop multiple times (think round trips)
/// - time: is either arrival or departure
pub type TripAtStopTimeMap<TT: TripType> = HashMap<(TT::Id, LocalStopId, u32), DateTime<Utc>>;

pub struct AnyTripAtStopTime {
    one_off: TripAtStopTimeMap<OneOff>,
    recurring: TripAtStopTimeMap<Recurring>,
}

impl AnyTripAtStopTime {
    fn get(
        &self,
        trip_id: &AnyTripId,
        stop_id: &LocalStopId,
        visit_idx: &u32,
    ) -> Option<&DateTime<Utc>> {
        match trip_id {
            AnyTripId::Recurring(trip_id) => self.recurring.get(&(*trip_id, *stop_id, *visit_idx)),
            AnyTripId::OneOff(trip_id) => self.one_off.get(&(*trip_id, *stop_id, *visit_idx)),
        }
    }
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
