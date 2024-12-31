use std::fmt;
use std::fmt::Formatter;
use crate::types::StopId;

#[derive(thiserror::Error, Debug)]
pub struct UnknownStopIdError(pub StopId);

impl fmt::Display for UnknownStopIdError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Unknown Stop ID {}", self.0)
    }
}