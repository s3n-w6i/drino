use chrono::Duration;
use common::types::StopId;
use crate::journey::Leg;
use crate::transfers::{TransferError, TransferProvider};

pub struct NoOpTransferProvider;

impl TransferProvider for NoOpTransferProvider {
    fn lower_bound_duration(&self, _start: StopId, _end: StopId) -> Result<Duration, TransferError> {
        unimplemented!()
    }

    fn duration(&self, _start: StopId, _end: StopId) -> Result<Duration, TransferError> {
        unimplemented!()
    }

    fn transfers_from(&self, _start: &StopId) -> Vec<StopId> {
        vec![]
    }

    fn transfers_between(&self, _start: StopId, _end: StopId) -> Result<Vec<Leg>, TransferError> {
        Ok(vec![])
    }
}