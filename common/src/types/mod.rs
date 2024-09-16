pub mod dataset;

// a continuous stop id
// "continuous" means that if we have n stops, all ids are from 0,...,n-1 and no number in that range
// is unused
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct StopId(pub u32);

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct LineId(pub u32);

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct TripId(pub u32);

// Sequence number
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct SeqNum(pub u32);