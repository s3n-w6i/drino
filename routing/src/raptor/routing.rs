use async_trait::async_trait;
use chrono::{DateTime, Duration, TimeDelta, Utc};
use hashbrown::HashSet;
use itertools::Itertools;
use std::cmp::min;
use std::iter::Skip;

use crate::algorithm::{AllEarliestArrival, AllRange, EarliestArrival, EarliestArrivalOutput, Journey, MultiQueryResult, QueryError, QueryResult, Range, RangeOutput, Single, SingleEarliestArrival, SingleRange};
use crate::raptor::state::RaptorState;
use crate::raptor::RaptorAlgorithm;
use crate::transfers::TransferProvider;
use common::types::{LineId, SeqNum, StopId, TripId};

const INFINITY: DateTime<Utc> = DateTime::<Utc>::MAX_UTC;

impl RaptorAlgorithm {
    fn earliest_trip(&self, line: LineId, stop: StopId, after: DateTime<Utc>) -> Option<TripId> {
        if let Some(trips) = self.trips_by_line_and_stop.get(&(line, stop)) {
            trips.into_iter().find_map(|(date, trip)| {
                if *date >= after {
                    Some(*trip)
                } else { None }
            })
        } else {
            None
        }
    }

    fn build_queue(&self, marked_stops: &Vec<StopId>) -> Vec<(LineId, StopId)> {
        let empty_line_set = HashSet::new();

        let mut queue: Vec<(LineId, StopId)> = Vec::new();

        for stop_a in marked_stops {
            let lines_serving_stop = self.lines_by_stops.get(stop_a)
                .unwrap_or(&empty_line_set);
            // foreach line serving marked_stop (stop a)
            for (line, seq_num_a) in lines_serving_stop {
                let other_stops = self.stops_by_line.get(line)
                    .expect(&format!("Line {line:?} is in lines_by_stops, so it must also be in stops_by_line."));
                // for any stop b that is also on the line
                for (seq_num_b, stop_b) in other_stops.iter().enumerate() {
                    let queue_position = queue.iter().position(|item| item == &(*line, *stop_b));
                    if let Some(position) = queue_position {
                        let seq_num_b = SeqNum(seq_num_b as u32);
                        // if other_stop comes after marked_stop on that line
                        if seq_num_a < &seq_num_b {
                            queue[position] = (*line, *stop_a);
                        }
                    } else {
                        queue.push((*line, *stop_a));
                    }
                }
            }
        }

        // TODO: Solve this more elegantly
        queue = queue.into_iter().unique().collect();

        queue
    }

    fn stops_on_line_after(&self, line: &LineId, stop: &StopId) -> Skip<std::slice::Iter<StopId>> {
        // Get all stops on that line that comes after stop_id (including stop_id)
        let stops_on_line = self.stops_by_line.get(line).unwrap();
        let a_stop_idx_on_line = stops_on_line.iter().position(|x| x == stop)
            .expect(&format!("Expected Stop with ID {stop:?} to be on line {line:?}"));
        let stops_on_line_after = stops_on_line.into_iter().skip(a_stop_idx_on_line);

        // stop_id itself is first in line of the stops
        debug_assert!(
            &stops_on_line_after.clone().collect::<Vec<&StopId>>()[0] == &stop,
            "Line {line:?} does not include stop {stop:?} as a stop after {stop:?}",
        );

        stops_on_line_after.into_iter()
    }

    fn run(
        &self,
        start: StopId,
        target: Option<StopId>,
        departure: DateTime<Utc>,
    ) -> QueryResult<RaptorState> {
        let mut state = RaptorState::init(self.stops.len(), start, departure);
        let mut marked_stops: Vec<StopId> = vec![start];

        // Increase the number of legs per round
        // foreach k <- 1,2,... do
        while !marked_stops.is_empty() {
            // increment k and set up this round
            state.new_round();

            // queue is called "Q" in the original paper
            let queue = self.build_queue(&marked_stops);

            // unmark previously marked stops
            // In the original paper, this is done for each element of marked_stops individually
            // while iterating over them. This is a simplification (otherwise, it's complicated with
            // Rust ownership system)
            marked_stops.clear();

            // SECOND STAGE
            // Process each line (called "route" in the original paper).
            for (line, a_stop) in queue.iter() {
                let mut trip: Option<TripId> = None;

                for b_stop in self.stops_on_line_after(line, a_stop) {
                    // TODO: Fix funky date problems
                    // if t != ⊥ and ...
                    if let Some(trip) = trip {
                        let b_arrival = self.arrivals.get(&(trip, *b_stop))
                            .expect("trip is not None, so b_stop cannot be the trip's first stop. Therefore, it must have an arrival time.");
                        let best_b_arrival = state.best_arrival(b_stop).unwrap_or(&INFINITY);
                        let best_target_arrival = target.and_then(|target| {
                            state.best_arrival(&target)
                        }).unwrap_or(&INFINITY);

                        // taking the trip to b it is faster than not taking it
                        // ...and arr(t, pᵢ) < min{ τ*(pᵢ), τ*(pₜ) }
                        if b_arrival < min(best_b_arrival, best_target_arrival) {
                            let a_departure = self.departures.get(&(trip, *a_stop))
                                .expect(&format!("Expected departure for stop {:?} to exist on trip {:?}", a_stop, trip));
                            state.set_ride(*a_stop, *b_stop, *a_departure, *b_arrival, trip);
                            marked_stops.push(*b_stop);
                        }
                    }

                    let b_departure = trip.and_then(|trip| {
                        self.departures.get(&(trip, *b_stop))
                    }).unwrap_or(&INFINITY);

                    let prev_b_arrival = state.previous_tau(b_stop)
                        .expect("At this position, k is >= 1, so a previous tau must exist.");

                    // Initialize trip if its None. Also execute when we can catch an earlier trip
                    // of the same line at stop b.
                    if prev_b_arrival <= b_departure {
                        trip = self.earliest_trip(*line, *b_stop, *prev_b_arrival);
                    }
                }
            }

            // THIRD STAGE
            // Look at individual station-to-station transfers (like footpaths) and update
            // earliest_arrival when walking to a stop is faster than taking transit
            let tp = &self.transfer_provider;
            // foreach marked stop p
            for start in marked_stops.clone() {
                // foreach footpath (p, p') ∈ F
                for end in tp.transfers_from(&start) {
                    // This is the maximum amount of time a transfer will have to take in order to
                    // be faster
                    let max_duration = *state.tau(&end).unwrap_or(&INFINITY) - *state.tau(&start)
                        .expect("transfer start was in marked_stops, so it must have a tau value set");

                    // This if-clause checks if there is any chance this transfer is faster.
                    // For this approximation, we use a lower bound duration that is cheaper to
                    // calculate than an actual route and duration (at least for large distances)
                    let lower_bound_duration = tp.lower_bound_duration(start, end)?;
                    if lower_bound_duration < max_duration {
                        // Since we found a candidate, calculate the actual, precise duration it
                        // will take.
                        let actual_duration = tp.duration(start, end)?;
                        debug_assert!(
                            actual_duration >= lower_bound_duration,
                            "Actual duration must be greater than the lower bound."
                        );
                        
                        if actual_duration < max_duration {
                            state.set_transfer(start, end, actual_duration);
                        }
                    }

                    // mark p'
                    marked_stops.push(end);
                }
            }
        }

        //println!("at the end {:?}", state);

        Ok(state)
    }

    async fn run_range(
        &self,
        start: StopId,
        target: Option<StopId>,
        earliest_departure: DateTime<Utc>,
        range: TimeDelta,
    ) -> QueryResult<RangeOutput> {
        let last_departure = earliest_departure + range;

        // List of all journeys to all targets in the given time range
        let mut journeys = HashSet::new();

        let mut time = earliest_departure;
        while time <= last_departure {
            match self.run(start, target, time) {
                // There is a valid output of the earliest arrival query
                Ok(state) => {
                    match target {
                        // If we have a target (this is a one-to-one query)
                        Some(target) => {
                            let journey = state.backtrace(target, time)?;
                            journeys.insert(journey.clone());
                            time = journey.departure().unwrap_or(time) + Duration::seconds(1); // TODO: Find a better way than this hack
                        }
                        // We have no target (one-to-all query)
                        None => {
                            let new_journeys = self.backtrace_all(state, time)?;
                            journeys.extend(new_journeys.clone().into_iter().collect::<Vec<Journey>>());

                            let earliest_departing_journey = new_journeys.iter()
                                .filter_map(|journey| journey.departure())
                                .min();
                            time = earliest_departing_journey.unwrap_or(time) + Duration::seconds(1);
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
        Ok(
            self.stops.iter()
                .map(|stop| state.backtrace(*stop, departure))
                .collect::<QueryResult<Vec<Journey>>>()?
        )
    }
}

#[async_trait]
impl SingleEarliestArrival for RaptorAlgorithm {
    async fn query_ea(
        &self,
        EarliestArrival { start, departure }: EarliestArrival,
        Single { target }: Single,
    ) -> QueryResult<EarliestArrivalOutput> {
        let res_state = self.run(start, Some(target), departure)?;
        let journey = res_state.backtrace(target, departure)?;
        Ok(EarliestArrivalOutput { journey })
    }
}

#[async_trait]
impl SingleRange for RaptorAlgorithm {
    async fn query_range(&self, Range { start, earliest_departure, range }: Range, Single { target }: Single) -> QueryResult<RangeOutput> {
        let range_result = self.run_range(start, Some(target), earliest_departure, range).await?;
        Ok(range_result)
    }
}

#[async_trait]
impl AllEarliestArrival for RaptorAlgorithm {
    async fn query_ea_all(&self, EarliestArrival { start, departure }: EarliestArrival) -> MultiQueryResult<EarliestArrivalOutput> {
        let res_state = self.run(start, None, departure)?;
        let journeys = self.backtrace_all(res_state, departure)?;
        let result = journeys.into_iter()
            .map(|journey| EarliestArrivalOutput { journey })
            .collect();
        Ok(result)
    }
}

#[async_trait]
impl AllRange for RaptorAlgorithm {
    async fn query_range_all(&self, Range { earliest_departure, range, start }: Range) -> QueryResult<RangeOutput> {
        self.run_range(start, None, earliest_departure, range).await
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::earliest_arrival_tests;
    use crate::tests::{duration_stop3_stop4, generate_case_4};
    use crate::transfers::CrowFlyTransferProvider;
    use geo::Coord;
    use hashbrown::{HashMap, HashSet};

    earliest_arrival_tests!(RaptorAlgorithm);

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
            transfer_provider: CrowFlyTransferProvider::from(vec![
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 40.0, y: 0.0 },
                Coord { x: -40.0, y: 0.0 },
            ]),
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
                DateTime::<Utc>::from_timestamp(250 + duration_stop3_stop4().num_seconds(), 0).unwrap(),
            ],
            "Best arrivals was not as expected. {res:?}"
        );
        // The k value that is reached after finding a way to all other stops
        assert_eq!(res.k, todo!("determine k"));
        todo!("Check the state of raptor after execution");
        raptor.run(StopId(0), Some(StopId(4)), dep0).expect("expected this to work");
    }
}