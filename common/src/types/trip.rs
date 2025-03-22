use std::fmt::Debug;
use std::hash::Hash;
use chrono::NaiveDate;
use serde::Serialize;

/// The different types of trips:
/// - Recurring trips: This is the usual type of trip you'd know from a transit system. The trip
///   runs on a regular basis. In GTFS and here in drino, we model a service that runs at different
///   times of the same day as different recurring trips for easier handling. This means that, to
///   identify a unique trip, we only need a number and a date at which this trip starts (and not a
///   time of that day).
/// - One-off trips: They occur once at on a specific day, given by a fixed date.

pub trait TripType {
    type Id: Serialize + Clone + Copy + Hash + Eq;
}

pub struct Recurring;

impl TripType for Recurring {
    type Id = RecurringTripId;
}

#[derive(Serialize, Debug, Clone, Hash, Eq, PartialEq, Copy)]
pub struct RecurringTripId {
    base_id: u32,
    /// The day this instance of the trip starts on. It might happen that a trip runs longer than
    /// 24 hours, so this is the day of the first departure.
    starting_day: NaiveDate,
}

pub struct OneOff;

impl TripType for OneOff {
    type Id = OneOffTripId;
}

#[derive(Serialize, Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct OneOffTripId(pub u32);

#[derive(Serialize, Debug, Clone, Copy, Hash, Eq, PartialEq)]
#[serde(untagged)]
pub enum AnyTripId {
    Recurring(RecurringTripId),
    OneOff(OneOffTripId),
}

impl Into<AnyTripId> for RecurringTripId {
    fn into(self) -> AnyTripId {
        AnyTripId::Recurring(self)
    }
}

impl Into<AnyTripId> for OneOffTripId {
    fn into(self) -> AnyTripId {
        AnyTripId::OneOff(self)
    }
}
