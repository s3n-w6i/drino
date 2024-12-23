pub mod fixed_time;
pub mod crow_fly;
pub mod noop;

use std::fmt;
use std::fmt::Display;

use crate::journey::Leg;
use chrono::Duration;
use common::types::StopId;

pub trait TransferProvider {
    fn lower_bound_duration(&self, start: StopId, end: StopId) -> Result<Duration, TransferError>;
    fn duration(&self, start: StopId, end: StopId) -> Result<Duration, TransferError>;

    // All transfers that are possible from the starting station. Must not include the station itself.
    fn transfers_from(&self, start: &StopId) -> Vec<StopId>;
    fn transfers_between(&self, start: StopId, end: StopId) -> Result<Vec<Leg>, TransferError>;
}

#[derive(thiserror::Error, Debug)]
pub enum TransferError {
    StopNotFound,
    OutOfReach
}

impl Display for TransferError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err: &dyn Display = match self {
            TransferError::StopNotFound => &"Stop not found",
            TransferError::OutOfReach => &"Transfer is out of reach"
        };
        write!(f, "{}", err)
    }
}

#[cfg(test)]
mod tests {
    use crate::transfers::crow_fly::CrowFlyTransferProvider;
    use common::util::speed::MAX_WALKING_SPEED;
    use geo::Coord;
    use super::*;

    #[test]
    fn test_crow_fly_provider() {
        let coord_a = Coord { x: 48.0, y: 9.0 };
        let coord_b = Coord { x: 10.0, y: 42.0 };
        let provider = CrowFlyTransferProvider::from(vec![coord_a, coord_b]);

        let transfers_from_0 = provider.transfers_from(&StopId(0));
        assert!(transfers_from_0.contains(&StopId(1)));
        let transfers_from_1 = provider.transfers_from(&StopId(1));
        assert!(transfers_from_1.contains(&StopId(0)));
        
        assert_eq!(
            provider.transfers_between(StopId(0), StopId(1)).unwrap(),
            vec![
                Leg::Transfer {
                    start: StopId(0),
                    end: StopId(1),
                    duration: MAX_WALKING_SPEED.time_to_travel_distance(5216815f32),
                }
            ]
        );

        assert_eq!(
            provider.transfers_between(StopId(1), StopId(0)).unwrap(),
            vec![
                Leg::Transfer {
                    start: StopId(1),
                    end: StopId(0),
                    duration: MAX_WALKING_SPEED.time_to_travel_distance(5216815f32),
                }
            ]
        )
    }
}