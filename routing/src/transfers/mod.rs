pub mod fixed_time;

use std::fmt;
use std::fmt::Display;

use chrono::Duration;
use geo::{Coord, HaversineDistance, Point};
use itertools::Itertools;
use polars::error::PolarsError;
use polars::prelude::{col, LazyFrame};

use crate::algorithm::Leg;
use common::types::StopId;
use common::util::speed::{MAX_WALKING_SPEED, Speed};

pub trait TransferProvider {
    fn lower_bound_duration(&self, start: StopId, end: StopId) -> Result<Duration, TransferError>;
    fn duration(&self, start: StopId, end: StopId) -> Result<Duration, TransferError>;

    // All transfers that are possible from the starting station. Must not include the station itself.
    fn transfers_from(&self, start: &StopId) -> Vec<StopId>;
    fn transfers_between(&self, start: StopId, end: StopId) -> Result<Vec<Leg>, TransferError>;
}

/// A pretty stupid transfer provider, that calculates the duration by measuring the distance
/// between stops and then going that distance in a straight line.
/// It basically always underestimates how long it takes.
#[derive(Clone)]
pub struct CrowFlyTransferProvider {
    stop_coords: Vec<Coord<f32>>,
    speed: Speed,
}

impl TransferProvider for CrowFlyTransferProvider {
    fn lower_bound_duration(&self, start: StopId, end: StopId) -> Result<Duration, TransferError> {
        let Some(start) = self.stop_coords.get(start.0 as usize) else { return Err(TransferError::StopNotFound); };
        let Some(end) = self.stop_coords.get(end.0 as usize) else { return Err(TransferError::StopNotFound); };

        let distance_meters = Point::from(*start).haversine_distance(&Point::from(*end));

        Ok(self.speed.time_to_travel_distance(distance_meters))
    }

    fn duration(&self, start: StopId, end: StopId) -> Result<Duration, TransferError> {
        self.lower_bound_duration(start, end)
    }

    fn transfers_from(&self, start: &StopId) -> Vec<StopId> {
        (0u32..self.stop_coords.len() as u32)
            // Return as Stop Ids, not as u32
            .map(|x| StopId(x))
            // Don't return the starting station itself
            .filter(|x| x != start)
            .collect()
    }

    fn transfers_between(&self, start: StopId, end: StopId) -> Result<Vec<Leg>, TransferError> {
        Ok(vec![
            Leg::Transfer { start, end, duration: self.duration(start, end)? }
        ])
    }
}

impl From<Vec<Coord<f32>>> for CrowFlyTransferProvider {
    fn from(stop_coords: Vec<Coord<f32>>) -> Self {
        Self { stop_coords, speed: MAX_WALKING_SPEED }
    }
}

impl CrowFlyTransferProvider {
    pub fn from_stops(stops_frame: LazyFrame) -> Result<Self, PolarsError> {
        let stop_lats = stops_frame.clone()
            .select(&[col("lat")])
            .collect()?.column("lat")?
            .f32()?.to_vec();
        let stop_lons = stops_frame.clone()
            .select(&[col("lon")])
            .collect()?.column("lon")?
            .f32()?.to_vec();

        let coords = stop_lats.into_iter().zip(stop_lons)
            .map(|(lat, lng)| {
                Coord { x: lat.unwrap(), y: lng.unwrap() }
            })
            .collect_vec();

        Ok(<Self as From<Vec<Coord<f32>>>>::from(coords))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum TransferError {
    StopNotFound
}

impl Display for TransferError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err: &dyn Display = match self {
            TransferError::StopNotFound => &"Stop not found"
        };
        write!(f, "{}", err)
    }
}

#[cfg(test)]
mod tests {
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