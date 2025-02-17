use crate::algorithm::*;
use crate::journey::Journey;
use crate::raptor::state::RaptorState;
use crate::raptor::{LocalStopId, RaptorAlgorithm};
use crate::transfers::TransferError;
use chrono::{DateTime, Duration, TimeDelta, Utc};
use common::types::{LineId, SeqNum, StopId, TripId};
use common::util::time::INFINITY;
use hashbrown::HashSet;
use itertools::Itertools;

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

    fn build_queue(&self, marked_stops: &HashSet<LocalStopId>) -> HashSet<(LineId, (LocalStopId, u32))> {
        let mut queue: HashSet<(LineId, (LocalStopId, u32))> = HashSet::new();

        for stop_a in marked_stops {
            if let Some(lines_serving_stop) = self.lines_by_stops.get(stop_a) {
                // foreach line serving marked_stop (stop a)
                for (line, seq_num_a) in lines_serving_stop {
                    let other_stops = self.stops_by_line.get(line)
                        .unwrap_or_else(|| panic!(
                            "Line {line:?} is in lines_by_stops, so it must also be in stops_by_line."
                        ));
                    // for any stop b that is also on the line
                    for (seq_num_b, (stop_b, visit_idx)) in other_stops.iter().enumerate() {
                        let seq_num_b = SeqNum(seq_num_b as u32);
                        // if other_stop comes after marked_stop on that line
                        if queue.contains(&(*line, (*stop_b, *visit_idx))) && seq_num_a < &seq_num_b {
                            queue.remove(&(*line, (*stop_b, *visit_idx)));
                            queue.insert((*line, (*stop_b, *visit_idx)));
                        } else {
                            queue.insert((*line, (*stop_b, *visit_idx)));
                        }
                    }
                }
            }
        }

        queue
    }

    fn stops_on_line_after(&self, line: &LineId, stop: &LocalStopId, visit_idx: &u32) -> impl Iterator<Item=&(LocalStopId, u32)> {
        // Get all stops on that line that comes after stop_id (including stop_id)
        let stops_on_line = self.stops_by_line.get(line).unwrap();
        let a_stop_idx_on_line = stops_on_line.iter()
            .position(|x| x == &(*stop, *visit_idx))
            .unwrap_or_else(|| panic!( // used instead of expect for performance
                "Expected Stop with ID {stop:?} to be on line {line:?}. But this line has only these stops: {stops_on_line:?}"
            ));
        let stops_on_line_after = stops_on_line.iter().skip(a_stop_idx_on_line);

        #[cfg(debug_assertions)] {
            // stop_id itself is first in line of the stops
            debug_assert!(
                stops_on_line_after.clone().collect_vec()[0].0 == *stop,
                "Line {line:?} does not include stop {stop:?} as a stop after {stop:?}",
            );
        }

        stops_on_line_after
    }

    fn run(
        &self,
        start: LocalStopId,
        departure: DateTime<Utc>,
    ) -> QueryResult<RaptorState> {
        let mut state = RaptorState::init(self.num_stops(), start, departure, &self.stop_mapping);
        let mut marked_stops: HashSet<LocalStopId> = HashSet::from([start]);

        // Increase the number of legs per round
        // foreach k <- 1,2,... do
        while !marked_stops.is_empty() {
            // increment k and set up this round
            state.new_round();
            debug_assert!(state.k > 0, "k starts at 1");

            // FIRST STAGE: Build queue of lines and stops to scan
            // queue is called "Q" in the original paper
            let queue = self.build_queue(&marked_stops);
            debug_assert!(!queue.is_empty(), "Queue must not be empty, since termination condition was not met");

            // unmark previously marked stops
            // In the original paper, this is done for each element of marked_stops individually
            // while iterating over them in `build_queue`. This is a simplification (otherwise, it's
            // complicated with Rust's ownership system)
            marked_stops.clear();

            // SECOND STAGE: Scan lines
            // Process each line (called "route" in the original paper).
            for (line, (a_stop, a_visit_idx)) in queue.iter() {
                // Option<(stop_id, visit_idx)>
                let mut boarding: Option<(StopId, u32)> = None;
                let mut trip: Option<TripId> = None;

                for (b_stop, b_visit_idx) in self.stops_on_line_after(line, a_stop, a_visit_idx) {
                    // TODO: Fix funky date problems
                    // if t != ⊥ and ...
                    if let Some(trip) = trip {
                        let b_arrival = self.arrivals.get(&(trip, *b_stop, *b_visit_idx))
                            .unwrap_or_else(|| panic!(
                                "Expected arrival for stop {b_stop:?} (visit {b_visit_idx}) to exist on trip {trip:?}"
                            ));
                        let best_b_arrival = state.best_arrival(b_stop);

                        // taking the trip to b it is faster than not taking it
                        // ...and arr(t, pᵢ) < τ*(pᵢ)
                        if b_arrival < best_b_arrival {
                            let (boarding_stop, boarding_visit_idx) = boarding.expect("Boarding stop must not be None");
                            let boarding_departure = self.departures.get(&(trip, boarding_stop, boarding_visit_idx))
                                .unwrap_or_else(|| panic!(
                                    "Expected departure for stop {a_stop:?} (visit {boarding_visit_idx}) to exist on trip {trip:?}"
                                ));

                            state.set_ride(boarding_stop, *b_stop, *boarding_departure, *b_arrival, trip);
                            marked_stops.insert(*b_stop);
                        }
                    }

                    let b_departure = trip.and_then(|trip| {
                        self.departures.get(&(trip, *b_stop, *b_visit_idx))
                    }).unwrap_or(&INFINITY);

                    let prev_b_arrival = state.previous_tau(b_stop);

                    // Initialize trip if its None. Also execute when we can catch an earlier trip
                    // of the same line at stop b.
                    if prev_b_arrival <= b_departure {
                        let next_trip = self.earliest_trip(*line, *b_stop, *prev_b_arrival);

                        if next_trip.is_some() {
                            trip = next_trip;
                            boarding = Some((*b_stop, *b_visit_idx));
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
                    let lower_bound_duration = transfer_provider.lower_bound_duration(start, end);
                    match lower_bound_duration {
                        Ok(lower_bound_duration) => {
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
                        },
                        Err(e) => {
                            match e {
                                TransferError::OutOfReach => {},
                                TransferError::StopNotFound => unreachable!("We only queried stops returned in provided transfer stops"),
                            }
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
        earliest_departure: DateTime<Utc>,
        range: TimeDelta,
    ) -> QueryResult<RangeOutput> {
        let last_departure = earliest_departure + range;

        // List of all journeys to all targets in the given time range
        let mut journeys = HashSet::new();

        let mut departure = earliest_departure;
        while departure <= last_departure {
            let res_after_departure = self.run(start, departure);

            match res_after_departure {
                // There is a valid output of the earliest arrival query
                Ok(state) => {
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
        let journeys = self.local_stop_ids()
            .map(|stop| state.backtrace(stop, departure))
            .filter_map(|res| res.ok())
            .collect_vec();

        if journeys.is_empty() {
            Err(QueryError::NoRouteFound)
        } else {
            Ok(journeys)
        }
    }
}

impl AllEarliestArrival for RaptorAlgorithm {
    fn query_ea_all(&self, EarliestArrival { start, earliest_departure }: EarliestArrival) -> MultiQueryResult<EarliestArrivalOutput> {
        let start = self.stop_mapping.translate_to_local(start);

        let res_state = self.run(start, earliest_departure)?;
        let journeys = self.backtrace_all(res_state, earliest_departure)?;
        let result = journeys.into_iter()
            .map(|journey| EarliestArrivalOutput { journey })
            .collect();
        Ok(result)
    }
}

impl AllRange for RaptorAlgorithm {
    fn query_range_all(&self, Range { earliest_departure, range, start }: Range) -> QueryResult<RangeOutput> {
        let start = self.stop_mapping.translate_to_local(start);

        self.run_range(start, earliest_departure, range)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::journey::Leg;
    use crate::raptor::tests::generate_case_4;
    use crate::raptor::StopMapping;
    use crate::transfers::fixed_time::FixedTimeTransferProvider;
    use common::util::duration;
    use hashbrown::{HashMap, HashSet};
    use ndarray::array;
    
    fn case1() -> RaptorAlgorithm {
        RaptorAlgorithm {
            stop_mapping: StopMapping(vec![0, 1].into_iter().map(|x| StopId(x)).collect()),
            stops_by_line: HashMap::from([
                (LineId(0), vec![(StopId(0), 0), (StopId(1), 0)])
            ]),
            lines_by_stops: HashMap::from([
                (StopId(0), HashSet::from([(LineId(0), SeqNum(0))])),
                (StopId(1), HashSet::from([(LineId(0), SeqNum(1))])),
            ]),
            arrivals: HashMap::from([
                ((TripId(0), StopId(1), 0), DateTime::<Utc>::from_timestamp(500, 0).unwrap())
            ]),
            departures: HashMap::from([
                ((TripId(0), StopId(0), 0), DateTime::<Utc>::from_timestamp(100, 0).unwrap())
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
    
    fn case2() -> RaptorAlgorithm {
        RaptorAlgorithm {
            stop_mapping: StopMapping(vec![0, 1, 2].into_iter().map(|x| StopId(x)).collect()),
            stops_by_line: HashMap::from([
                (LineId(0), vec![(StopId(0), 0), (StopId(1), 0)]),
                (LineId(1), vec![(StopId(1), 0), (StopId(2), 0)]),
            ]),
            lines_by_stops: HashMap::from([
                (StopId(0), HashSet::from([(LineId(0), SeqNum(0))])),
                (StopId(1), HashSet::from([(LineId(0), SeqNum(1)), (LineId(1), SeqNum(0))])),
                (StopId(2), HashSet::from([(LineId(1), SeqNum(1))])),
            ]),
            departures: HashMap::from([
                ((TripId(0), StopId(0), 0), DateTime::<Utc>::from_timestamp(100, 0).unwrap()),
                ((TripId(1), StopId(1), 0), DateTime::<Utc>::from_timestamp(1000, 0).unwrap()),
            ]),
            arrivals: HashMap::from([
                ((TripId(0), StopId(1), 0), DateTime::<Utc>::from_timestamp(500, 0).unwrap()),
                ((TripId(1), StopId(2), 0), DateTime::<Utc>::from_timestamp(1500, 0).unwrap()),
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
        }
    }

    fn case1_trip0_leg0() -> Leg {
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
        let raptor = case2();

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
        let res = raptor.run(StopId(0), dep0).unwrap();

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
                        (1, case1_trip0_leg0())
                    ])
                )
            ]),
            stop_mapping: &StopMapping(vec![StopId(0), StopId(1)])
        };

        let res = case1().backtrace_all(state, DateTime::UNIX_EPOCH).unwrap();

        assert_eq!(res, vec![Journey::from(vec![case1_trip0_leg0()])]);
    }

    /// 0 --Ride--> 1
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
        assert_eq!(res.journeys, HashSet::from([Journey::from( vec![case1_trip0_leg0()] )]));

        // query later, after missing the only connection there is
        let res = raptor.query_range_all(
            Range { earliest_departure: DateTime::<Utc>::from_timestamp(300, 0).unwrap(), range: Duration::weeks(42), start: StopId(0) },
        );
        assert!(matches!(res, Err(QueryError::NoRouteFound)));
    }

    ///   0 ---Ride--> 1
    ///   0 ---Ride--> 1 ---Ride--> 2
    #[tokio::test]
    async fn test_query_range_all_2() {
        let raptor = case2();

        let actual = raptor.query_range_all(
            Range { start: StopId(0), earliest_departure: DateTime::UNIX_EPOCH, range: Duration::seconds(100) },
        ).unwrap();
        
        let case2_trip0_leg0 = Leg::Ride {
            boarding_stop: StopId(0),
            alight_stop: StopId(1),
            boarding_time: DateTime::<Utc>::from_timestamp(100, 0).unwrap(),
            alight_time: DateTime::<Utc>::from_timestamp(500, 0).unwrap(),
            trip: TripId(0),
        };
        
        let expected = RangeOutput {
            journeys: HashSet::from([
                Journey::from(vec![
                    case2_trip0_leg0.clone(),
                    Leg::Ride {
                        boarding_stop: StopId(1),
                        alight_stop: StopId(2),
                        boarding_time: DateTime::<Utc>::from_timestamp(1000, 0).unwrap(),
                        alight_time: DateTime::<Utc>::from_timestamp(1500, 0).unwrap(),
                        trip: TripId(1),
                    },
                ]),
                Journey::from(vec![
                    case2_trip0_leg0
                ])
            ])
        };

        assert_eq!(actual, expected);
    }

    ///   0 ---Ride--> 1
    ///   0 ---Ride--> 1 ---Transfer--> 2
    ///   0 ---Ride--> 1 ---Transfer--> 2 ---Ride--> 3
    #[tokio::test]
    async fn test_query_range_all_3() {
        let duration_1_to_2 = Duration::seconds(10);

        let raptor = RaptorAlgorithm {
            stop_mapping: StopMapping(vec![0, 1, 2, 3].into_iter().map(|x| StopId(x)).collect()),
            stops_by_line: HashMap::from([
                (LineId(0), vec![(StopId(0), 0), (StopId(1), 0)]),
                (LineId(1), vec![(StopId(2), 0), (StopId(3), 0)]),
            ]),
            lines_by_stops: HashMap::from([
                (StopId(0), HashSet::from([(LineId(0), SeqNum(0))])),
                (StopId(1), HashSet::from([(LineId(0), SeqNum(1))])),
                (StopId(2), HashSet::from([(LineId(1), SeqNum(0))])),
                (StopId(3), HashSet::from([(LineId(1), SeqNum(1))])),
            ]),
            departures: HashMap::from([
                ((TripId(0), StopId(0), 0), DateTime::<Utc>::from_timestamp(100, 0).unwrap()),
                ((TripId(1), StopId(2), 0), DateTime::<Utc>::from_timestamp(1000, 0).unwrap()),
            ]),
            arrivals: HashMap::from([
                ((TripId(0), StopId(1), 0), DateTime::<Utc>::from_timestamp(500, 0).unwrap()),
                ((TripId(1), StopId(3), 0), DateTime::<Utc>::from_timestamp(1500, 0).unwrap()),
            ]),
            trips_by_line_and_stop: HashMap::from([
                ((LineId(0), StopId(0)), vec![(DateTime::<Utc>::from_timestamp(100, 0).unwrap(), TripId(0))]),
                ((LineId(1), StopId(2)), vec![(DateTime::<Utc>::from_timestamp(1000, 0).unwrap(), TripId(1))]),
            ]),
            transfer_provider: Box::new(FixedTimeTransferProvider {
                duration_matrix: array![
                        [Duration::zero(),   duration::INFINITY, duration::INFINITY, duration::INFINITY],
                        [duration::INFINITY, Duration::zero(),   duration_1_to_2,    duration::INFINITY],
                        [duration::INFINITY, duration_1_to_2,    Duration::zero(),   duration::INFINITY],
                        [duration::INFINITY, duration::INFINITY, duration::INFINITY, Duration::zero()  ],
                    ]
            }),
        };

        let actual = raptor.query_range_all(
            Range { start: StopId(0), earliest_departure: DateTime::UNIX_EPOCH, range: Duration::seconds(101) },
        ).unwrap();
        
        let case3_journey0_leg0 = Leg::Ride {
            boarding_stop: StopId(0),
            alight_stop: StopId(1),
            boarding_time: DateTime::<Utc>::from_timestamp(100, 0).unwrap(),
            alight_time: DateTime::<Utc>::from_timestamp(500, 0).unwrap(),
            trip: TripId(0),
        };
        let case3_journey0_leg1 = Leg::Transfer {
            start: StopId(1),
            end: StopId(2),
            duration: duration_1_to_2,
        };
        
        let expected = RangeOutput { journeys: HashSet::from([
            Journey::from(vec![case3_journey0_leg0.clone()]),
            Journey::from(vec![case3_journey0_leg0.clone(), case3_journey0_leg1.clone()]),
            Journey::from(vec![
                case3_journey0_leg0,
                case3_journey0_leg1,
                Leg::Ride {
                    boarding_stop: StopId(2),
                    alight_stop: StopId(3),
                    boarding_time: DateTime::<Utc>::from_timestamp(1000, 0).unwrap(),
                    alight_time: DateTime::<Utc>::from_timestamp(1500, 0).unwrap(),
                    trip: TripId(1),
                },
            ])
        ]) };

        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_query_earliest_4() {
        let dep20 = DateTime::<Utc>::from_timestamp(20, 0).unwrap();
        let dep220 = DateTime::<Utc>::from_timestamp(220, 0).unwrap();
        let dep110 = DateTime::<Utc>::from_timestamp(110, 0).unwrap();
        let dep310 = DateTime::<Utc>::from_timestamp(310, 0).unwrap();
        let dep0 = DateTime::<Utc>::from_timestamp(0, 0).unwrap();
        let dep400 = DateTime::<Utc>::from_timestamp(400, 0).unwrap();
        let dep490 = DateTime::<Utc>::from_timestamp(490, 0).unwrap();
        let dep150 = DateTime::<Utc>::from_timestamp(150, 0).unwrap();
        let dep550 = DateTime::<Utc>::from_timestamp(550, 0).unwrap();

        let arr100 = DateTime::<Utc>::from_timestamp(100, 0).unwrap();
        let arr300 = DateTime::<Utc>::from_timestamp(300, 0).unwrap();
        let arr150 = DateTime::<Utc>::from_timestamp(150, 0).unwrap();
        let arr350 = DateTime::<Utc>::from_timestamp(350, 0).unwrap();
        let arr200 = DateTime::<Utc>::from_timestamp(200, 0).unwrap();
        let arr600 = DateTime::<Utc>::from_timestamp(600, 0).unwrap();
        let arr700 = DateTime::<Utc>::from_timestamp(700, 0).unwrap();
        let arr250 = DateTime::<Utc>::from_timestamp(250, 0).unwrap();

        let raptor = generate_case_4();

        // 0 ---Ride(130_1)--> 3 ---Transfer--> 4
        // Takes 250s + 410s = 660s
        let actual = raptor.query_range_all(
            Range { start: StopId(0), earliest_departure: dep0, range: Duration::seconds(100) },
        ).unwrap();
        
        let case4_journey_0_leg0 = Leg::Ride {
            trip: TripId(130_1),
            boarding_stop: StopId(0),
            alight_stop: StopId(3),
            boarding_time: dep0,
            alight_time: arr250,
        };
        
        let expected = RangeOutput { journeys: HashSet::from([
            Journey::from(vec![case4_journey_0_leg0.clone()]),
            Journey::from(vec![
                case4_journey_0_leg0,
                Leg::Transfer {
                    start: StopId(3),
                    end: StopId(4),
                    duration: Duration::seconds(410),
                },
            ])
        ])};
        
        assert_eq!(actual, expected);

        // Start 1s second later than the last case. Now, we can't take 130_1 anymore, since it
        // departs at 0s. Instead, we have to take this slower connection:
        // 0@20s   ---Ride(100_1)-->   2@100s, 2@490s   ---Ride(120_2)-->   4@700s
        // this connection arrives well before the following, which would arrive at 710s instead
        // of 700s:
        // 0@20s   ---Ride(100_1)-->   3@300s   ---Transfer-->   4@710s
        let actual = raptor.query_range_all(
            Range { start: StopId(0), earliest_departure: DateTime::<Utc>::from_timestamp(1, 0).unwrap(), range: Duration::seconds(100) },
        ).unwrap();
        
        let case4_journey1_leg0 = Leg::Ride {
            trip: TripId(100_1),
            boarding_stop: StopId(0), alight_stop: StopId(2),
            boarding_time: dep20, alight_time: arr100,
        };
        
        let expected = RangeOutput { journeys: HashSet::from([
            Journey::from(vec![case4_journey1_leg0.clone()]),
            Journey::from(vec![
                case4_journey1_leg0,
                Leg::Ride {
                    trip: TripId(120_2),
                    boarding_stop: StopId(2), alight_stop: StopId(4),
                    boarding_time: dep490, alight_time: arr700,
                },
            ])
        ])};

        assert_eq!(actual, expected);

        // TODO: More cases
    }
}