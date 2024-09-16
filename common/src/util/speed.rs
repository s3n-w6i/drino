use chrono::{Duration, TimeDelta};

#[derive(Copy, Clone)]
pub struct Speed(pub f64); // in km/h

pub const MAX_WALKING_SPEED: Speed = Speed(10f64);

impl Speed {
    pub fn time_to_travel_distance(&self, meters: f32) -> Duration {
        let hours = (1.0 / self.0) * (meters as f64 / 1_000.0);
        TimeDelta::milliseconds((hours * 60.0 * 60.0 * 1_000.0) as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_speed_to_distance() {
        assert_eq!(Duration::seconds(36), Speed(10.0).time_to_travel_distance(100.));
        assert_eq!(Duration::seconds(18), Speed(200.0).time_to_travel_distance(1_000.));
    }
}