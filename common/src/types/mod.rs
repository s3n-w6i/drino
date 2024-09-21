use polars::datatypes::AnyValue;

pub mod dataset;

// a continuous stop id
// "continuous" means that if we have n stops, all ids are from 0,...,n-1 and no number in that range
// is unused
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct StopId(pub u32);

impl <'a> Into<AnyValue<'a>> for StopId {
    fn into(self) -> AnyValue<'a> {
        AnyValue::UInt32(self.0)
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

impl <'a> Into<AnyValue<'a>> for LineId {
    fn into(self) -> AnyValue<'a> {
        AnyValue::UInt32(self.0)
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

impl <'a> Into<AnyValue<'a>> for TripId {
    fn into(self) -> AnyValue<'a> {
        AnyValue::UInt32(self.0)
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

impl <'a> Into<AnyValue<'a>> for SeqNum {
    fn into(self) -> AnyValue<'a> {
        AnyValue::UInt32(self.0)
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