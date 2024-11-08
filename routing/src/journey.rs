use chrono::{DateTime, Duration, TimeDelta, Utc};
use common::types::{StopId, TripId};
use std::slice::Iter;
use itertools::Itertools;

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
    legs: Vec<Leg>,
}

impl Journey {

    fn new(legs: Vec<Leg>) -> Self {
        debug_assert!(legs.len() > 0, "A Journey must have at least one leg");
        
        if cfg!(debug_assertions) {
            // Check that the legs form a valid chain of stops: For each leg, the end location must
            // match the next leg's starting location
            let mut last_transfer_stop = legs.first().unwrap().end();
            for leg in legs.iter().skip(1) {
                debug_assert!(last_transfer_stop == leg.start());
                last_transfer_stop = leg.end();
            }
            
            // Check that there are no cycles in the journey
            // A journey must not go back to a stop where we came from, since then it is longer than
            // it needs to be. Since we already checked that end == start of next, only check start
            // locations for uniqueness.
            let stops = legs.iter().map(|leg| leg.start());
            let stops_unique = stops.clone().all_unique();
            debug_assert!(
                stops_unique,
                "Expected stops of journey to be unique. Instead, stops {:?} are visited twice. All legs: {:?}",
                stops.clone().duplicates().collect_vec(),
                legs
            );
        }
        
        Self { legs }
    }

    pub(crate) fn legs(&self) -> Iter<Leg> {
        self.legs.iter()
    }

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

    pub(crate) fn departure_stop(&self) -> &StopId {
        let first_leg = self.legs.first().expect("Journey must have at least one leg");
        first_leg.start()
    }

    pub(crate) fn arrival_stop(&self) -> &StopId {
        let last_leg = self.legs.last().expect("Journey must have at least one leg");
        last_leg.end()
    }
}

impl From<Vec<Leg>> for Journey {
    fn from(legs: Vec<Leg>) -> Self {
        Self::new(legs)
    }
}