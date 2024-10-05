use chrono::Duration;
use common::types::StopId;
use crate::algorithm::Leg;
use crate::transfers::{TransferError, TransferProvider};

/// Don't calculate any transfer time, instead return precalculated/hard-coded values from lookup table.
/// Useful for testing or if everything is already calculated.
/// Due to its simplicity it is also possible to define asymmetric durations (a -> b different time than b -> a)
#[derive(Clone)]
pub struct FixedTimeTransferProvider {
    pub(crate) duration_matrix: ndarray::Array2<Duration>
}

impl TransferProvider for FixedTimeTransferProvider {
    fn lower_bound_duration(&self, start: StopId, end: StopId) -> Result<Duration, TransferError> {
        self.duration(start, end)
    }

    fn duration(&self, start: StopId, end: StopId) -> Result<Duration, TransferError> {
        Ok(self.duration_matrix[[start.0 as usize, end.0 as usize]])
    }

    fn transfers_from(&self, start: &StopId) -> Vec<StopId> {
        debug_assert!(self.duration_matrix.is_square());
        (0..self.duration_matrix.ncols())
            .map(|id| StopId(id as u32))
            // Don't return the starting station itself
            .filter(|x| x != start)
            .collect()
    }

    fn transfers_between(&self, start: StopId, end: StopId) -> Result<Vec<Leg>, TransferError> {
        Ok(vec![
            Leg::Transfer {
                start, end, duration: self.duration(start, end)?
            }
        ])
    }
}