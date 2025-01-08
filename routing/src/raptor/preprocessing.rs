use crate::algorithm::{
    PreprocessInit, PreprocessingError, PreprocessingInput, PreprocessingResult,
};
use crate::direct_connections::DirectConnections;
use crate::raptor::{
    GlobalStopId, LinesByStopMap, RaptorAlgorithm, StopMapping, StopsByLineMap, TripAtStopTimeMap,
    TripsByLineAndStopMap,
};
use crate::transfers::crow_fly::CrowFlyTransferProvider;
use chrono::DateTime;
use common::types::{LineId, StopId, TripId};
use common::util::time::INFINITY;
use itertools::{izip, Itertools};
use polars::error::PolarsError;
use polars::prelude::*;
use std::ops::{BitAnd, BitOr};

impl PreprocessInit for RaptorAlgorithm {
    fn preprocess(
        input: PreprocessingInput,
        save_to_disk: bool,
    ) -> PreprocessingResult<RaptorAlgorithm> {
        if save_to_disk {
            unimplemented!()
        }

        let direct_connections = DirectConnections::try_from(input.clone())?;
        Self::preprocess(input, direct_connections)
    }
}

impl RaptorAlgorithm {
    pub fn preprocess(
        PreprocessingInput { stops, .. }: PreprocessingInput,
        DirectConnections {
            expanded_lines,
            line_progressions,
            ..
        }: DirectConnections,
    ) -> PreprocessingResult<RaptorAlgorithm> {
        let stops_vec: Vec<GlobalStopId> = stops.clone()
            .select(&[col("stop_id")]).collect()?
            .column("stop_id")?.u32()?
            .to_vec().into_iter()
            .filter_map(|x| x.map(StopId))
            .collect();

        let stop_mapping = StopMapping(stops_vec);

        let (stops_by_line, lines_by_stops) = {
            let mut stops_by_line = hashbrown::HashMap::default();
            let mut lines_by_stops = hashbrown::HashMap::default();

            let [line_ids, global_stop_ids, sequence_numbers] = line_progressions.get_columns()
            else {
                return Err(PreprocessingError::Polars(PolarsError::ColumnNotFound(
                    "".into(),
                )));
            };

            let [line_ids, global_stop_ids, sequence_numbers] = [
                line_ids.u32()?,
                global_stop_ids.u32()?,
                sequence_numbers.u32()?,
            ];

            for (line_id, global_stop_id, seq_num) in
                izip!(line_ids, global_stop_ids, sequence_numbers)
            {
                let line_id = line_id.unwrap().into();
                let global_stop_id = global_stop_id.unwrap().into();
                let local_stop_id = stop_mapping.translate_to_local(global_stop_id);
                let seq_num = seq_num.unwrap().into();

                let stops_by_line_entry = stops_by_line.entry(line_id).or_insert(vec![]);
                let visit_idx = stops_by_line_entry.iter()
                    .filter(|(stop_id, _)| stop_id == &local_stop_id)
                    .count() as u32;
                stops_by_line_entry.push((local_stop_id, visit_idx));

                lines_by_stops.entry(local_stop_id).or_insert(hashbrown::HashSet::new())
                    .insert((line_id, seq_num));
            }

            Ok::<(StopsByLineMap, LinesByStopMap), PreprocessingError>((
                stops_by_line,
                lines_by_stops,
            ))
        }?;
        debug_assert!(stop_mapping.0.len() == lines_by_stops.len());

        let lines = expanded_lines.clone().select([
            "line_id",
            "stop_id",
            "stop_sequence",
            "trip_id",
            "arrival_time",
            "departure_time",
        ])?;

        let (arrivals, departures) = {
            let sorted_lines = lines.clone().sort(
                ["line_id", "trip_id", "stop_sequence"],
                SortMultipleOptions::default()
                    .with_maintain_order(false)
                    .with_order_descending(false),
            )?;
            let [_line_ids, global_stop_ids, _sequence_numbers, trip_ids, arrival_times, departure_times] =
                sorted_lines.get_columns()
            else {
                return Err(PreprocessingError::Polars(PolarsError::ColumnNotFound(
                    "".into(),
                )));
            };

            let [global_stop_ids, trip_ids] = [global_stop_ids.u32()?, trip_ids.u32()?];
            let arrival_times = arrival_times.duration()?;
            let departure_times = departure_times.duration()?;

            debug_assert!(arrival_times.time_unit() == TimeUnit::Milliseconds);
            debug_assert!(departure_times.time_unit() == TimeUnit::Milliseconds);

            let mut arrivals = hashbrown::HashMap::default();
            let mut departures = hashbrown::HashMap::default();
            for (trip_id, global_stop_id, arrival_time, departure_time) in
                izip!(trip_ids, global_stop_ids, arrival_times.iter(), departure_times.iter())
            {
                let trip_id = TripId(trip_id.unwrap());
                let global_stop_id = StopId(global_stop_id.unwrap());
                let local_stop_id = stop_mapping.translate_to_local(global_stop_id);
                let arrival_time = arrival_time.unwrap();
                let departure_time = departure_time.unwrap();

                // TODO: Fix date time handling
                let arrival_time = DateTime::from_timestamp_millis(arrival_time).unwrap();
                let departure_time = DateTime::from_timestamp_millis(departure_time).unwrap();

                // Determine the how-many-th time this stop is visited. For most, this will be zero,
                // so this rather inefficient code should do.
                let mut visit_idx = 0u32;
                while arrivals.get(&(trip_id, local_stop_id, visit_idx)).is_some() {
                    visit_idx += 1;
                }

                if !cfg!(debug_assertions) {
                    unsafe {
                        arrivals.insert_unique_unchecked((trip_id, local_stop_id, visit_idx), arrival_time);
                        departures.insert_unique_unchecked((trip_id, local_stop_id, visit_idx), departure_time);
                    }
                } else {
                    arrivals.insert((trip_id, local_stop_id, visit_idx), arrival_time);
                    departures.insert((trip_id, local_stop_id, visit_idx), departure_time);
                }
            }

            #[cfg(debug_assertions)]
            {
                // Assert that no arrival at a stop is after the departure
                arrivals.iter()
                    .for_each(|((trip, stop, visit_idx), arrival)| {
                        let departure = departures
                            .get(&(*trip, *stop, *visit_idx))
                            .unwrap_or(&INFINITY);
                        debug_assert!(
                            arrival <= departure,
                            "Departure at stop {stop:?} must be after arrival. Issue found on {trip:?}"
                        );
                    });
            }

            Ok::<(TripAtStopTimeMap, TripAtStopTimeMap), PreprocessingError>((arrivals, departures))
        }?;

        let trips_by_line_and_stop_df = lines.clone().lazy()
            .sort(
                ["departure_time"],
                SortMultipleOptions::default().with_maintain_order(false),
            )
            .group_by(&[col("line_id"), col("stop_id")])
            .agg(&[col("trip_id"), col("departure_time")])
            .collect()?;
        let [line_ids, global_stop_ids, trips_ids, departures_times] =
            trips_by_line_and_stop_df.get_columns()
        else {
            return Err(PreprocessingError::Polars(PolarsError::ColumnNotFound(
                "".into(),
            )));
        };
        let line_ids = line_ids.u32()?;
        let global_stop_ids = global_stop_ids.u32()?;
        let trips_ids = trips_ids.list()?;
        let departures_times = departures_times.list()?;

        let mut trips_by_line_and_stop: TripsByLineAndStopMap = hashbrown::HashMap::default();

        for (line_id, global_stop_id, trips, departures) in
            izip!(line_ids, global_stop_ids, trips_ids, departures_times)
        {
            let trips = trips.unwrap();
            let departures = departures.unwrap();
            let departures_trips = departures
                .duration()?.iter()
                .zip(trips.u32()?)
                .filter_map(|(departure, trip)| {
                    departure.map(|departure| {
                        // TODO: Fix date conversion
                        (
                            DateTime::from_timestamp_millis(departure).unwrap(),
                            TripId(trip.unwrap()),
                        )
                    })
                })
                .collect();
            let line_id = LineId(line_id.unwrap());
            let global_stop_id = StopId(global_stop_id.unwrap());
            let local_stop_id = stop_mapping.translate_to_local(global_stop_id);

            trips_by_line_and_stop.insert((line_id, local_stop_id), departures_trips);
        }

        #[cfg(debug_assertions)]
        {
            // Assert monotonous increase in departure time within a trip
            for ((line, _), departures) in trips_by_line_and_stop.iter() {
                // We can only check increase for trips that have at least two stops
                if departures.len() >= 2 {
                    let (last_departure_time, last_trip) = departures.first().unwrap();
                    for (departure_time, trip) in departures.iter().skip(1) {
                        debug_assert!(
                            departure_time >= last_departure_time,
                            "Expected departure time ({departure_time}) to not be smaller than previous ({last_departure_time}) in trips_by_line_and_stop. Offending Trip: {trip:?} (compared to {last_trip:?}) on line {line:?}. Excerpt from lines DF:\n{}\nExcerpt from trips_by_line_and_stop:\n{:#?}",
                            {
                                let line_mask = lines.column("line_id")?.as_materialized_series().equal(line.0)?;
                                let trip_series = lines.column("trip_id")?.as_materialized_series();
                                let trip_mask = trip_series.equal(trip.0)?;
                                let last_trip_mask = trip_series.equal(last_trip.0)?;
                                lines.clone().filter(&line_mask.bitand(trip_mask.bitor(last_trip_mask)))
                            }?,
                            trips_by_line_and_stop.iter()
                                .filter(|((l, _), _)| l == line)
                                .collect_vec()
                        );
                    }
                }
            }

            // Assert that stop sequences of lines match across trips_by_line_and_stop and stops_by_line
            let stops_by_line_a = &stops_by_line;
            let stops_by_line_b = &trips_by_line_and_stop.clone().into_keys().into_group_map();

            stops_by_line_a.iter().for_each(|(line, stops_a)| {
                // Sort so that vecs match
                // Unique removes double stops, since stops are deduplicated in stops_by_line_b 
                let stops_a = stops_a.iter()
                    .map(|(stop_id, _)| stop_id)
                    .unique().sorted().collect_vec();
                let stops_b = stops_by_line_b.get(line)
                    .unwrap_or_else(|| panic!("Expected line {line:?} to exist in trips_by_line_and_stop"))
                    .iter().sorted()
                    .collect_vec();

                debug_assert!(
                    stops_a == stops_b,
                    "Stops don't match for {line:?}. stops_by_line: {:?}\ntrips_by_line_and_stop: {:?}",
                    stops_a, stops_b
                );
            });
        }

        Ok(Self {
            stop_mapping,
            stops_by_line,
            lines_by_stops,
            arrivals,
            departures,
            trips_by_line_and_stop,
            transfer_provider: Box::new(CrowFlyTransferProvider::from_stops(stops)?),
        })
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
        let departure_times: Series = (0..15).into_iter().collect::<Series>()
            .cast(&DataType::Duration(TimeUnit::Milliseconds)).unwrap();
        
        // Stop sequences of lines:
        // [s:0, s:1, s:2, s:3]          Trips: [t:0]
        // [s:2, s:3, s:4, s:5]          Trips: [t:1]
        // [s:3, s:4]                    Trips: [t:2]
        // [s:0, s:1, s:2, s:3, s:4]     Trips: [t:3]
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

        let preprocessing_out =
            <RaptorAlgorithm as PreprocessInit>::preprocess(preprocessing_in, false).unwrap();

        assert!(list_eq(
            &preprocessing_out.stop_mapping.0,
            &vec![0u32, 1, 2, 3, 4, 5].into_iter().map(|x| StopId(x)).collect())
        );
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
