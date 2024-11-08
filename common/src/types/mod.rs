use std::fmt::{Debug, Display, Formatter};
use chrono::{DateTime, NaiveDate, Utc};
use polars::datatypes::AnyValue;

pub mod dataset;

// a continuous stop id
// "continuous" means that if we have n stops, all ids are from 0,...,n-1 and no number in that range
// is unused
#[derive(Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct StopId(pub u32);

impl Display for StopId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("s:{}", self.0))
    }
}

impl Debug for StopId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("s:{}", self.0))
    }
}

impl <'a> From<StopId> for AnyValue<'a> {
    fn from(value: StopId) -> AnyValue<'a> {
        AnyValue::UInt32(value.0)
    }
}

impl <'a> TryFrom<AnyValue<'a>> for StopId {
    type Error = ();

    fn try_from(value: AnyValue<'a>) -> Result<Self, Self::Error> {
        match value {
            AnyValue::UInt32(value) => Ok(Self(value)),
            _ => Err(())
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct LineId(pub u32);

impl <'a> From<LineId> for AnyValue<'a> {
    fn from(value: LineId) -> AnyValue<'a> {
        AnyValue::UInt32(value.0)
    }
}

impl <'a> TryFrom<AnyValue<'a>> for LineId {
    type Error = ();

    fn try_from(value: AnyValue<'a>) -> Result<Self, Self::Error> {
        match value {
            AnyValue::UInt32(value) => Ok(Self(value)),
            _ => Err(())
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct TripId(pub u32);

impl <'a> From<TripId> for AnyValue<'a> {
    fn from(value: TripId) -> AnyValue<'a> {
        AnyValue::UInt32(value.0)
    }
}

impl <'a> TryFrom<AnyValue<'a>> for TripId {
    type Error = ();

    fn try_from(value: AnyValue<'a>) -> Result<Self, Self::Error> {
        match value {
            AnyValue::UInt32(value) => Ok(Self(value)),
            _ => Err(())
        }
    }
}

// Sequence number
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct SeqNum(pub u32);

impl <'a> From<SeqNum> for AnyValue<'a> {
    fn from(value: SeqNum) -> AnyValue<'a> {
        AnyValue::UInt32(value.0)
    }
}

impl <'a> TryFrom<AnyValue<'a>> for SeqNum {
    type Error = ();

    fn try_from(value: AnyValue<'a>) -> Result<Self, Self::Error> {
        match value {
            AnyValue::UInt32(value) => Ok(Self(value)),
            _ => Err(())
        }
    }
}