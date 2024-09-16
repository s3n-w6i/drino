use chrono::DateTime;
use hashbrown::{HashMap, HashSet};
use itertools::izip;
use polars::error::{ErrString, PolarsError};
use polars::prelude::{col, IntoLazy, SortMultipleOptions};
use crate::algorithm::{PreprocessingError, PreprocessingInput, PreprocessingResult, PreprocessInit};
use crate::direct_connections::DirectConnections;
use crate::transfers::CrowFlyTransferProvider;
use common::types::{LineId, SeqNum, StopId, TripId};
use crate::raptor::RaptorAlgorithm;

impl RaptorAlgorithm {
    pub fn preprocess(input: PreprocessingInput, DirectConnections { lines, .. }: DirectConnections) -> PreprocessingResult<RaptorAlgorithm> {        
        let stops = input.stops.clone()
            .select(&[col("stop_id")])
            .collect()?.column("stop_id")?
            .u32()?.to_vec()
            .into_iter().filter_map(|x| x)
            .map(|x| StopId(x))
            .collect();

        let lines = lines.clone()
            .select(["line_id", "stop_id", "stop_sequence", "trip_id", "arrival_time", "departure_time"])?
            .sort(["line_id", "trip_id", "stop_sequence"], SortMultipleOptions::default()
                .with_maintain_order(false)
                .with_order_descending(false))?;

        let [line_ids, stop_ids, sequence_numbers, trip_ids, arrival_times, departure_times] =
            lines.get_columns()
            else { return Err(PreprocessingError::Polars(PolarsError::ColumnNotFound(ErrString::from("")))); };
        let line_ids = line_ids.u32()?;
        let stop_ids = stop_ids.u32()?;
        let sequence_numbers = sequence_numbers.u32()?;
        let trip_ids = trip_ids.u32()?;
        let arrival_times = arrival_times.duration()?;
        let departure_times = departure_times.duration()?;

        let mut stops_by_line = HashMap::new();
        for (idx, line_id) in line_ids.into_iter().enumerate() {
            let line_id = LineId(line_id.unwrap());
            let stop_id = StopId(idx as u32);

            stops_by_line.entry(line_id).or_insert(vec![])
                .push(stop_id);
        }

        let mut lines_by_stops = HashMap::new();
        for (idx, stop_id) in stop_ids.into_iter().enumerate() {
            let stop_id = StopId(stop_id.unwrap());
            let line_id = LineId(line_ids.get(idx).unwrap());
            let sequence_number = SeqNum(sequence_numbers.get(idx).unwrap());
            let value = (line_id, sequence_number);

            lines_by_stops.entry(stop_id).or_insert(HashSet::new())
                .insert(value);
        }

        let mut arrivals = HashMap::new();
        let mut departures = HashMap::new();
        for (idx, trip_id) in trip_ids.iter().enumerate() {
            let trip_id = TripId(trip_id.unwrap());
            let stop_id = StopId(idx as u32);
            // TODO: Fix date time handling
            let arrival_time = DateTime::from_timestamp_millis(
                arrival_times.get(idx).unwrap()
            ).unwrap();
            let departure_time = DateTime::from_timestamp_millis(
                departure_times.get(idx).unwrap()
            ).unwrap();
            arrivals.insert((trip_id, stop_id), arrival_time);
            departures.insert((trip_id, stop_id), departure_time);
        }


        let trips_by_line_and_stop_df = lines.clone().lazy()
            .group_by(&[col("line_id"), col("stop_id")])
            .agg(&[col("trip_id"), col("departure_time")])
            .collect()?;
        let [line_ids, stop_ids, trips_ids, departures_times] = trips_by_line_and_stop_df.get_columns()
            else { return Err(PreprocessingError::Polars(PolarsError::ColumnNotFound(ErrString::from("")))); };
        let line_ids = line_ids.u32()?;
        let stop_ids = stop_ids.u32()?;
        let trips_ids = trips_ids.list()?;
        let departures_times = departures_times.list()?;

        let mut trips_by_line_and_stop = HashMap::new();

        for (line_id, stop_id, trips, departures) in izip!(line_ids, stop_ids, trips_ids, departures_times) {
            let trips = trips.unwrap();
            let departures = departures.unwrap();
            let departures_trips = departures.duration()?.iter().zip(trips.u32()?)
                .filter_map(|(departure, trip)| {
                    if let Some(departure) = departure {
                        // TODO: Fix date conversion
                        Some((
                            DateTime::from_timestamp_millis(departure).unwrap(),
                            TripId(trip.unwrap())
                        ))
                    } else { None }
                });

            trips_by_line_and_stop.insert(
                (LineId(line_id.unwrap()), StopId(stop_id.unwrap())),
                departures_trips.collect(),
            );
        }

        Ok(Self {
            stops,
            stops_by_line,
            lines_by_stops,
            arrivals,
            departures,
            trips_by_line_and_stop,
            transfer_provider: CrowFlyTransferProvider::from_stops(input.stops.clone())?
        })
    }
}

impl PreprocessInit for RaptorAlgorithm {
    fn preprocess(input: PreprocessingInput) -> PreprocessingResult<RaptorAlgorithm> {
        let direct_connections = DirectConnections::try_from(input.clone())?;
        Self::preprocess(input, direct_connections)
    }
}



#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use polars::df;
    use polars::frame::DataFrame;
    use polars::prelude::*;

    use super::*;

    #[test]
    fn test_preprocessing() {
        let departure_times: Series = [0; 15]
            .into_iter().collect::<Series>()
            .cast(&DataType::Duration(TimeUnit::Milliseconds)).unwrap();

        let preprocessing_in = PreprocessingInput {
            services: DataFrame::empty().lazy(),
            stops: df!(
                "stop_id" => &[0u32, 1, 2, 3, 4, 5],
                "lat"     => &[0.0f32, 1.0, 5.0, -10.0, 80.0, -42.0 ],
                "lon"     => &[0.0f32, 1.0, 5.0, -10.0, 80.0, -42.0 ],
            ).unwrap().lazy(),
            trips: df!(
                "trip_id" => &[0u32, 1, 2, 3],
            ).unwrap().lazy(),
            stop_times: df!(
                "trip_id"        => &[0u32, 0, 0,  1,  0,  1, 1, 1,  2, 2,  3, 3, 3, 3, 3],
                "stop_id"        => &[0u32, 1, 2,  2,  3,  3, 4, 5,  3, 4,  0, 1, 2, 3, 4],
                "arrival_time"   => departure_times.clone(),
                "departure_time" => departure_times.clone(),
                "stop_sequence"  => &[0u32, 1, 2, 3, 4, 5, 6, 7, 8, 10, 11, 12, 13, 14, 15]
            ).unwrap().lazy(),
        };

        let preprocessing_out = <RaptorAlgorithm as PreprocessInit>::preprocess(preprocessing_in).unwrap();

        assert!(list_eq(&preprocessing_out.stops, &vec![0u32, 1, 2, 3, 4, 5].into_iter().map(|x| StopId(x)).collect()));
        // TODO: Test all of preprocessing_out
    }

    fn list_eq<T>(a: &Vec<T>, b: &Vec<T>) -> bool
        where
            T: PartialEq + Ord,
    {
        a.iter().sorted().collect_vec();
        b.iter().sorted().collect_vec();

        a == b
    }
}