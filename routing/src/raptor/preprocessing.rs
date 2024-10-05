use crate::algorithm::{PreprocessInit, PreprocessingError, PreprocessingInput, PreprocessingResult};
use crate::direct_connections::DirectConnections;
use crate::raptor::RaptorAlgorithm;
use crate::transfers::CrowFlyTransferProvider;
use chrono::DateTime;
use common::types::{LineId, SeqNum, StopId, TripId};
use hashbrown::{HashMap, HashSet};
use indicatif::MultiProgress;
use itertools::izip;
use polars::error::PolarsError;
use polars::prelude::{col, IntoLazy, SortMultipleOptions};

impl RaptorAlgorithm {
    pub fn preprocess(
        PreprocessingInput { stops, .. }: PreprocessingInput,
        DirectConnections { lines, .. }: DirectConnections,
    ) -> PreprocessingResult<RaptorAlgorithm> {
        let stops_vec = stops.clone()
            .select(&[col("stop_id")])
            .collect()?.column("stop_id")?
            .u32()?.to_vec()
            .into_iter().filter_map(|x| x)
            .map(|x| StopId(x))
            .collect();

        let lines = lines.clone()
            .select(["line_id", "stop_id", "stop_sequence", "trip_id", "arrival_time", "departure_time"])?
            .sort(
                ["line_id", "trip_id", "stop_sequence"],
                SortMultipleOptions::default()
                    .with_maintain_order(false)
                    .with_order_descending(false),
            )?;

        let [line_ids, stop_ids, sequence_numbers, trip_ids, arrival_times, departure_times] =
            lines.get_columns()
        else { return Err(PreprocessingError::Polars(PolarsError::ColumnNotFound("".into()))); };
        
        let line_ids = line_ids.u32()?;
        let stop_ids = stop_ids.u32()?;
        let sequence_numbers = sequence_numbers.u32()?;
        let trip_ids = trip_ids.u32()?;
        let arrival_times = arrival_times.duration()?;
        let departure_times = departure_times.duration()?;

        let mut stops_by_line = HashMap::new();
        for (line_id, stop_id) in izip!(line_ids, stop_ids) {
            let line_id = LineId(line_id.unwrap());
            let stop_id = StopId(stop_id.unwrap());

            stops_by_line.entry(line_id).or_insert(vec![])
                .push(stop_id);
        }

        let mut lines_by_stops = HashMap::new();
        for (stop_id, line_id, seq_num) in izip!(stop_ids, line_ids, sequence_numbers) {
            let stop_id = StopId(stop_id.unwrap());
            let line_id = LineId(line_id.unwrap());
            let seq_num = SeqNum(seq_num.unwrap());

            lines_by_stops.entry(stop_id).or_insert(HashSet::new())
                .insert((line_id, seq_num));
        }

        let mut arrivals = HashMap::new();
        let mut departures = HashMap::new();
        for (trip_id, stop_id, arrival_time, departure_time) in izip!(trip_ids, stop_ids, arrival_times.iter(), departure_times.iter()) {
            let trip_id = TripId(trip_id.unwrap());
            let stop_id = StopId(stop_id.unwrap());
            let arrival_time = arrival_time.unwrap();
            let departure_time = departure_time.unwrap();
            // TODO: Fix date time handling
            let arrival_time = DateTime::from_timestamp_millis(arrival_time).unwrap();
            let departure_time = DateTime::from_timestamp_millis(departure_time).unwrap();
            arrivals.insert((trip_id, stop_id), arrival_time);
            departures.insert((trip_id, stop_id), departure_time);
        }


        let trips_by_line_and_stop_df = lines.clone().lazy()
            .group_by(&[col("line_id"), col("stop_id")])
            .agg(&[col("trip_id"), col("departure_time")])
            .collect()?;
        let [line_ids, stop_ids, trips_ids, departures_times] = trips_by_line_and_stop_df.get_columns()
        else { return Err(PreprocessingError::Polars(PolarsError::ColumnNotFound("".into()))); };
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
            stops: stops_vec,
            stops_by_line,
            lines_by_stops,
            arrivals,
            departures,
            trips_by_line_and_stop,
            transfer_provider: Box::new(CrowFlyTransferProvider::from_stops(stops)?),
        })
    }
}

impl<'a> PreprocessInit for RaptorAlgorithm {
    fn preprocess(input: PreprocessingInput, _: Option<&MultiProgress>) -> PreprocessingResult<RaptorAlgorithm> {
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

        let preprocessing_out = <RaptorAlgorithm as PreprocessInit>::preprocess(preprocessing_in, None).unwrap();

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
    
    // TODO: More test cases. This one test passed, despite the function being wrong!
}