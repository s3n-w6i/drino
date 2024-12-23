use crate::journey::Leg;
use crate::transfers::{TransferError, TransferProvider};
use chrono::Duration;
use common::types::StopId;
use common::util::speed::{Speed, MAX_WALKING_DURATION, MAX_WALKING_SPEED};
use geo::{Coord, Distance, Haversine, Point};
use itertools::Itertools;
use polars::error::PolarsError;
use polars::prelude::{col, LazyFrame};

/// A pretty stupid transfer provider, that calculates the duration by measuring the distance
/// between stops and then going that distance in a straight line.
/// It basically always underestimates how long it takes.
#[derive(Clone)]
pub struct CrowFlyTransferProvider {
    stop_coords: Vec<Coord<f32>>,
    speed: Speed,
    max_duration: Duration,
}

impl TransferProvider for CrowFlyTransferProvider {
    fn lower_bound_duration(&self, start: StopId, end: StopId) -> Result<Duration, TransferError> {
        let Some(start) = self.stop_coords.get(start.0 as usize) else { return Err(TransferError::StopNotFound); };
        let Some(end) = self.stop_coords.get(end.0 as usize) else { return Err(TransferError::StopNotFound); };

        let distance_meters = Haversine::distance(Point::from(*start), Point::from(*end));
        
        let time = self.speed.time_to_travel_distance(distance_meters);
        
        if time <= self.max_duration {
            Ok(time)
        } else {
            Err(TransferError::OutOfReach)
        }
    }

    fn duration(&self, start: StopId, end: StopId) -> Result<Duration, TransferError> {
        self.lower_bound_duration(start, end)
    }

    fn transfers_from(&self, start: &StopId) -> Vec<StopId> {
        (0u32..self.stop_coords.len() as u32)
            // Return as Stop Ids, not as u32
            .map(StopId)
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
        Self {
            stop_coords, speed: MAX_WALKING_SPEED, max_duration: MAX_WALKING_DURATION
        }
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