use crate::algorithm::*;
use crate::raptor::state::RaptorState;
use crate::raptor::RaptorAlgorithm;
use chrono::{DateTime, Duration, TimeDelta, Utc};
use common::types::{LineId, SeqNum, StopId, TripId};
use common::util::time::INFINITY;
use hashbrown::HashSet;
use itertools::Itertools;
use std::cmp::min;
use std::iter::Skip;
use crate::journey::Journey;

impl RaptorAlgorithm {
    /// Selects the earliest trip of a line, that departs at `stop` after a given time
    fn earliest_trip(&self, line: LineId, stop: StopId, after: DateTime<Utc>) -> Option<TripId> {
        self.trips_by_line_and_stop
            .get(&(line, stop))
            .and_then(|trips| {
                trips.iter().find_map(|(departure, trip)| {
                    if *departure >= after {
                        Some(*trip)
                    } else { None }
                })
            })
    }

    fn build_queue(&self, marked_stops: &HashSet<StopId>) -> HashSet<(LineId, StopId)> {
        let mut queue: HashSet<(LineId, StopId)> = HashSet::new();

        for stop_a in marked_stops {
            if let Some(lines_serving_stop) = self.lines_by_stops.get(stop_a) {
                // foreach line serving marked_stop (stop a)
                for (line, seq_num_a) in lines_serving_stop {
                    let other_stops = self.stops_by_line.get(line)
                        .unwrap_or_else(|| panic!(
                            "Line {line:?} is in lines_by_stops, so it must also be in stops_by_line."
                        ));
                    // for any stop b that is also on the line
                    for (seq_num_b, stop_b) in other_stops.iter().enumerate() {
                        let seq_num_b = SeqNum(seq_num_b as u32);
                        // if other_stop comes after marked_stop on that line
                        if queue.contains(&(*line, *stop_b)) && seq_num_a < &seq_num_b {
                            queue.remove(&(*line, *stop_b));
                            queue.insert((*line, *stop_a));
                        } else {
                            queue.insert((*line, *stop_a));
                        }
                    }
                }
            }
        }

        queue
    }

    fn stops_on_line_after(&self, line: &LineId, stop: &StopId) -> Skip<std::slice::Iter<StopId>> {
        // Get all stops on that line that comes after stop_id (including stop_id)
        let stops_on_line = self.stops_by_line.get(line).unwrap();
        let a_stop_idx_on_line = stops_on_line.iter().position(|x| x == stop)
            .unwrap_or_else(|| panic!( // used instead of expect for performance
                "Expected Stop with ID {stop:?} to be on line {line:?}. But this line has only these stops: {stops_on_line:?}"
            ));
        let stops_on_line_after = stops_on_line.iter().skip(a_stop_idx_on_line);

        #[cfg(debug_assertions)] {
            // stop_id itself is first in line of the stops
            debug_assert!(
                stops_on_line_after.clone().collect::<Vec<&StopId>>()[0] == stop,
                "Line {line:?} does not include stop {stop:?} as a stop after {stop:?}",
            );
        }

        stops_on_line_after.into_iter()
    }

    fn run(
        &self,
        start: StopId,
        target: Option<StopId>,
        departure: DateTime<Utc>,
    ) -> QueryResult<RaptorState> {
        let mut state = RaptorState::init(self.stops.len(), start, departure);
        let mut marked_stops: HashSet<StopId> = HashSet::from([start]);

        // Increase the number of legs per round
        // foreach k <- 1,2,... do
        while !marked_stops.is_empty() {
            // increment k and set up this round
            state.new_round();
            debug_assert!(state.k > 0, "k starts at 1");

            // FIRST STAGE: Build queue of lines and stops to scan
            // queue is called "Q" in the original paper
            let queue = self.build_queue(&marked_stops);
            debug_assert!(!queue.is_empty());

            // unmark previously marked stops
            // In the original paper, this is done for each element of marked_stops individually
            // while iterating over them in `build_queue`. This is a simplification (otherwise, it's
            // complicated with Rust's ownership system)
            marked_stops.clear();

            // SECOND STAGE: Scan lines
            // Process each line (called "route" in the original paper).
            for (line, a_stop) in queue.iter() {
                let mut boarding_stop: Option<StopId> = None;
                let mut trip: Option<TripId> = None;

                for b_stop in self.stops_on_line_after(line, a_stop) {
                    // TODO: Fix funky date problems
                    // if t != ⊥ and ...
                    if let Some(trip) = trip {
                        let b_arrival = self.arrivals.get(&(trip, *b_stop))
                            .unwrap_or(&INFINITY);
                        let best_b_arrival = state.best_arrival(b_stop);
                        let best_target_arrival = target.map(|target| {
                            state.best_arrival(&target)
                        }).unwrap_or(&INFINITY);

                        // taking the trip to b it is faster than not taking it
                        // ...and arr(t, pᵢ) < min{ τ*(pᵢ), τ*(pₜ) }
                        if b_arrival < min(best_b_arrival, best_target_arrival) {
                            let boarding_stop = boarding_stop.expect("Boarding stop must not be None");
                            let boarding_stop_departure = self.departures.get(&(trip, boarding_stop))
                                .unwrap_or_else(|| panic!(
                                    "Expected departure for stop {a_stop:?} to exist on trip {trip:?}"
                                ));

                            state.set_ride(boarding_stop, *b_stop, *boarding_stop_departure, *b_arrival, trip);
                            marked_stops.insert(*b_stop);
                        }
                    }

                    let b_departure = trip.and_then(|trip| {
                        self.departures.get(&(trip, *b_stop))
                    }).unwrap_or(&INFINITY);

                    let prev_b_arrival = state.previous_tau(b_stop);

                    // Initialize trip if its None. Also execute when we can catch an earlier trip
                    // of the same line at stop b.
                    if prev_b_arrival <= b_departure {
                        let next_trip = self.earliest_trip(*line, *b_stop, *prev_b_arrival);

                        if next_trip.is_some() {
                            trip = next_trip;
                            boarding_stop = Some(*b_stop);
                        }
                    }
                }
            }

            // THIRD STAGE: Scan transfers
            // Look at individual station-to-station transfers (like footpaths) and update
            // best_arrival when walking to a stop is faster than taking transit
            let transfer_provider = &self.transfer_provider;
            // foreach marked stop p
            for start in marked_stops.clone() {
                // foreach footpath (p, p') ∈ F
                for end in transfer_provider.transfers_from(&start) {
                    // This is the maximum amount of time a transfer will have to take in order to
                    // be faster
                    let max_duration = *state.tau(&end).unwrap_or(&INFINITY) - *state.tau(&start)
                        .expect("transfer start was in marked_stops, so it must have a tau value set");

                    // This if-clause checks if there is any chance this transfer is faster.
                    // For this approximation, we use a lower bound duration that is cheaper to
                    // calculate than an actual route and duration (at least for large distances)
                    let lower_bound_duration = transfer_provider.lower_bound_duration(start, end)?;
                    if lower_bound_duration < max_duration {
                        // Since we found a candidate, calculate the actual, precise duration it
                        // will take.
                        let actual_duration = transfer_provider.duration(start, end)?;
                        debug_assert!(
                            actual_duration >= lower_bound_duration,
                            "Actual duration must be greater than the lower bound."
                        );

                        if actual_duration < max_duration {
                            state.set_transfer(start, end, actual_duration);
                        }
                    }

                    // mark p'
                    marked_stops.insert(end);
                }
            }
        }

        Ok(state)
    }

    fn run_range(
        &self,
        start: StopId,
        target: Option<StopId>,
        earliest_departure: DateTime<Utc>,
        range: TimeDelta,
    ) -> QueryResult<RangeOutput> {
        let last_departure = earliest_departure + range;

        // List of all journeys to all targets in the given time range
        let mut journeys = HashSet::new();

        let mut departure = earliest_departure;
        while departure <= last_departure {
            let res_after_departure = self.run(start, target, departure);

            match res_after_departure {
                // There is a valid output of the earliest arrival query
                Ok(state) => {
                    match target {
                        // If we have a target (this is a one-to-one query)
                        Some(target) => {
                            if let Ok(journey) = state.backtrace(target, departure) {
                                let journey_departure = journey.departure().unwrap_or(departure);

                                if journey_departure <= last_departure {
                                    journeys.insert(journey.clone());
                                }
                                departure = journey_departure + Duration::seconds(1); // TODO: Find a better way than this hack
                            } else { break; }
                        }
                        // We have no specific target (this is a one-to-all query)
                        None => {
                            let new_journeys = self.backtrace_all(state, departure)
                                .unwrap_or_default()
                                .into_iter()
                                .filter(|j| { j.departure().unwrap_or(departure) <= last_departure });

                            journeys.extend(new_journeys.clone());

                            let earliest_departure = new_journeys
                                .filter_map(|journey| journey.departure())
                                .min();

                            if let Some(earliest_departure) = earliest_departure {
                                departure = earliest_departure + Duration::seconds(1);
                            } else {
                                // There is no earliest departure, so there is no departure at all
                                // after this point in time
                                break;
                            }
                        }
                    };
                }
                // There were no (more) trips found: stop searching
                Err(QueryError::NoRouteFound) => break,
                // Propagate errors up to the caller
                Err(other_err) => {
                    return Err(other_err);
                }
            }
        }

        // Check if any journeys were found at all
        if journeys.is_empty() {
            return Err(QueryError::NoRouteFound);
        }

        Ok(RangeOutput { journeys })
    }

    fn backtrace_all(&self, state: RaptorState, departure: DateTime<Utc>) -> QueryResult<Vec<Journey>> {
        let journeys = self.stops.iter()
            .map(|stop| state.backtrace(*stop, departure))
            .filter_map(|res| res.ok())
            .collect_vec();

        if journeys.is_empty() {
            Err(QueryError::NoRouteFound)
        } else {
            Ok(journeys)
        }
    }
}

impl SingleEarliestArrival for RaptorAlgorithm {
    fn query_ea(
        &self,
        EarliestArrival { start, departure }: EarliestArrival,
        Single { target }: Single,
    ) -> QueryResult<EarliestArrivalOutput> {
        let res_state = self.run(start, Some(target), departure)?;
        let journey = res_state.backtrace(target, departure)?;
        Ok(EarliestArrivalOutput { journey })
    }
}

impl SingleRange for RaptorAlgorithm {
    fn query_range(&self, Range { start, earliest_departure, range }: Range, Single { target }: Single) -> QueryResult<RangeOutput> {
        self.run_range(start, Some(target), earliest_departure, range)
    }
}

impl AllEarliestArrival for RaptorAlgorithm {
    fn query_ea_all(&self, EarliestArrival { start, departure }: EarliestArrival) -> MultiQueryResult<EarliestArrivalOutput> {
        let res_state = self.run(start, None, departure)?;
        let journeys = self.backtrace_all(res_state, departure)?;
        let result = journeys.into_iter()
            .map(|journey| EarliestArrivalOutput { journey })
            .collect();
        Ok(result)
    }
}

impl AllRange for RaptorAlgorithm {
    fn query_range_all(&self, Range { earliest_departure, range, start }: Range) -> QueryResult<RangeOutput> {
        self.run_range(start, None, earliest_departure, range)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::earliest_arrival_tests;
    use crate::tests::generate_case_4;
    use crate::transfers::fixed_time::FixedTimeTransferProvider;
    use common::util::duration;
    use hashbrown::{HashMap, HashSet};
    use ndarray::array;
    use crate::journey::Leg;

    earliest_arrival_tests!(RaptorAlgorithm);

    fn case1() -> RaptorAlgorithm {
        RaptorAlgorithm {
            stops: vec![0, 1].into_iter().map(|x| StopId(x)).collect(),
            stops_by_line: HashMap::from([
                (LineId(0), vec![StopId(0), StopId(1)])
            ]),
            lines_by_stops: HashMap::from([
                (StopId(0), HashSet::from([(LineId(0), SeqNum(0))])),
                (StopId(1), HashSet::from([(LineId(0), SeqNum(1))])),
            ]),
            arrivals: HashMap::from([
                ((TripId(0), StopId(1)), DateTime::<Utc>::from_timestamp(500, 0).unwrap())
            ]),
            departures: HashMap::from([
                ((TripId(0), StopId(0)), DateTime::<Utc>::from_timestamp(100, 0).unwrap())
            ]),
            trips_by_line_and_stop: HashMap::from([
                ((LineId(0), StopId(0)), vec![(DateTime::<Utc>::from_timestamp(100, 0).unwrap(), TripId(0))]),
            ]),
            transfer_provider: Box::new(FixedTimeTransferProvider {
                duration_matrix: array![
                    [Duration::zero(), Duration::max_value(),],
                    [Duration::max_value(), Duration::zero(),],
                ]
            }),
        }
    }

    fn case1_journey0_leg0() -> Leg {
        Leg::Ride {
            trip: TripId(0),
            boarding_stop: StopId(0),
            alight_stop: StopId(1),
            boarding_time: DateTime::<Utc>::from_timestamp(100, 0).unwrap(),
            alight_time: DateTime::<Utc>::from_timestamp(500, 0).unwrap(),
        }
    }

    #[test]
    fn test_earliest_trip_function() {
        let raptor = RaptorAlgorithm {
            stops: vec![0, 1, 2].into_iter().map(|x| StopId(x)).collect(),
            stops_by_line: HashMap::from([
                (LineId(0), vec![StopId(0), StopId(1)]),
                (LineId(1), vec![StopId(1), StopId(2)]),
            ]),
            lines_by_stops: HashMap::from([
                (StopId(0), HashSet::from([(LineId(0), SeqNum(0))])),
                (StopId(1), HashSet::from([(LineId(0), SeqNum(1)), (LineId(1), SeqNum(0))])),
                (StopId(2), HashSet::from([(LineId(1), SeqNum(1))])),
            ]),
            departures: HashMap::from([
                ((TripId(0), StopId(0)), DateTime::<Utc>::from_timestamp(100, 0).unwrap()),
                ((TripId(1), StopId(1)), DateTime::<Utc>::from_timestamp(1000, 0).unwrap()),
            ]),
            arrivals: HashMap::from([
                ((TripId(0), StopId(1)), DateTime::<Utc>::from_timestamp(500, 0).unwrap()),
                ((TripId(1), StopId(2)), DateTime::<Utc>::from_timestamp(1500, 0).unwrap()),
            ]),
            trips_by_line_and_stop: HashMap::from([
                ((LineId(0), StopId(0)), vec![(DateTime::<Utc>::from_timestamp(100, 0).unwrap(), TripId(0))]),
                ((LineId(1), StopId(1)), vec![(DateTime::<Utc>::from_timestamp(1000, 0).unwrap(), TripId(1))]),
            ]),
            transfer_provider: Box::new(FixedTimeTransferProvider {
                duration_matrix: array![
                    [Duration::zero(), duration::INFINITY, duration::INFINITY,],
                    [duration::INFINITY, Duration::zero(), duration::INFINITY,],
                    [duration::INFINITY, duration::INFINITY, Duration::zero(),],
                ]
            }),
        };

        assert_eq!(
            raptor.earliest_trip(LineId(0), StopId(0), DateTime::<Utc>::from_timestamp(0, 0).unwrap()),
            Some(TripId(0))
        );
        assert_eq!(
            raptor.earliest_trip(LineId(0), StopId(0), DateTime::<Utc>::from_timestamp(100, 0).unwrap()),
            Some(TripId(0))
        );
        assert_eq!(
            raptor.earliest_trip(LineId(0), StopId(0), DateTime::<Utc>::from_timestamp(100, 1).unwrap()),
            None
        );

        // Stop 2 is not served by Line 0
        assert_eq!(
            raptor.earliest_trip(LineId(0), StopId(2), DateTime::<Utc>::from_timestamp(0, 1).unwrap()),
            None
        );
        // Stop 2 is the terminus of Line 1, so there is no trip departing from there at any time
        assert_eq!(
            raptor.earliest_trip(LineId(1), StopId(2), DateTime::<Utc>::from_timestamp(0, 1).unwrap()),
            None
        );
    }

    #[test]
    fn test_final_state() {
        let dep0 = DateTime::<Utc>::from_timestamp(0, 0).unwrap();

        let raptor = generate_case_4();
        let res = raptor.run(StopId(0), None, dep0).unwrap();

        // The k value that is reached after finding a way to all other stops
        // It's 3 since going to 1 or 4 takes two legs, going to 2 or 3 just takes one leg, and we
        // do an additional round for finding out that nothing changed in the last round
        // => marked_stops is empty.
        assert_eq!(res.k, 3);

        assert_eq!(res.k_arrivals.len(), res.k + 1);

        assert_eq!(
            res.best_arrivals,
            vec![
                // Stop 0: We're already here (it's the starting point) -> should take no time
                DateTime::<Utc>::from_timestamp(0, 0).unwrap(),
                // Stop 1: Fastest way is 0 --100_1--> 2 --101_1--> 1
                DateTime::<Utc>::from_timestamp(150, 0).unwrap(),
                // Stop 2: Fastest way is 0 --100_1--> 2
                DateTime::<Utc>::from_timestamp(100, 0).unwrap(),
                // Stop 3: Fastest way is 0 --130_1--> 3
                DateTime::<Utc>::from_timestamp(250, 0).unwrap(),
                // Stop 4: Fastest way is 0 --130_1--> 3 --Walk--> 4 (120_2 departs too late. By then, it's faster to walk)
                DateTime::<Utc>::from_timestamp(250 + 410, 0).unwrap(),
            ],
            "Best arrivals was not as expected. {res:?}"
        );

        // TODO: Test connection index

        for i in 0u32..3 {
            let stop_id = StopId(i);
            let res_single = raptor.run(StopId(0), Some(StopId(4)), dep0).expect("expected this to work");
            assert_eq!(
                res.best_arrivals[i as usize], res_single.best_arrivals[i as usize],
                "Best arrivals were different between one to one and one to all for StopId(0) to {stop_id:?}"
            );
        }
    }

    #[test]
    fn test_backtrace_all() {
        let state = RaptorState {
            k: 2,
            k_arrivals: vec![
                vec![
                    DateTime::UNIX_EPOCH,
                    INFINITY
                ],
                vec![
                    DateTime::UNIX_EPOCH,
                    DateTime::<Utc>::from_timestamp(500, 0).unwrap(),
                ],
                vec![
                    DateTime::UNIX_EPOCH,
                    DateTime::<Utc>::from_timestamp(500, 0).unwrap(),
                ],
            ],
            best_arrivals: vec![
                DateTime::UNIX_EPOCH,
                DateTime::<Utc>::from_timestamp(500, 0).unwrap(),
            ],
            connection_index: HashMap::from([
                (
                    StopId(1),
                    HashMap::from([
                        (1, case1_journey0_leg0())
                    ])
                )
            ]),
        };

        let res = case1().backtrace_all(state, DateTime::UNIX_EPOCH).unwrap();

        assert_eq!(res, vec![Journey::from(vec![case1_journey0_leg0()])]);
    }

    #[test]
    fn test_query_range_single_1() {
        let raptor = case1();

        // Query a too short range starting from 0
        let res = raptor.query_range(
            Range { earliest_departure: DateTime::UNIX_EPOCH, range: Duration::seconds(98), start: StopId(0) },
            Single { target: StopId(1) },
        );
        assert!(matches!(res, Err(QueryError::NoRouteFound)));

        // Query a longer range starting from 0
        let res = raptor.query_range(
            Range { earliest_departure: DateTime::UNIX_EPOCH, range: Duration::seconds(101), start: StopId(0) },
            Single { target: StopId(1) },
        ).unwrap();
        assert_eq!(res.journeys, HashSet::from([Journey::from(vec![case1_journey0_leg0()])]));

        // query later, after missing the only connection there is
        let res = raptor.query_range(
            Range { earliest_departure: DateTime::<Utc>::from_timestamp(300, 0).unwrap(), range: Duration::weeks(42), start: StopId(0) },
            Single { target: StopId(1) },
        );
        assert!(matches!(res, Err(QueryError::NoRouteFound)));
    }

    #[test]
    fn test_query_range_all_1() {
        let raptor = case1();

        // Query a too short range starting from 0
        let res = raptor.query_range_all(
            Range { earliest_departure: DateTime::UNIX_EPOCH, range: Duration::seconds(98), start: StopId(0) },
        );
        assert!(matches!(res, Err(QueryError::NoRouteFound)));

        // Query a longer range starting from 0
        let res = raptor.query_range_all(
            Range { earliest_departure: DateTime::UNIX_EPOCH, range: Duration::seconds(101), start: StopId(0) },
        ).unwrap();
        assert_eq!(res.journeys, HashSet::from([Journey::from( vec![case1_journey0_leg0()] )]));

        // query later, after missing the only connection there is
        let res = raptor.query_range_all(
            Range { earliest_departure: DateTime::<Utc>::from_timestamp(300, 0).unwrap(), range: Duration::weeks(42), start: StopId(0) },
        );
        assert!(matches!(res, Err(QueryError::NoRouteFound)));
    }
}