use std::cmp::min;

use crate::algorithm::Leg;
use crate::algorithm::QueryError::NoRouteFound;
use common::types::StopId;

use super::*;

// HashMap<Stop_id, HashMap<k, Connection>>
type ConnectionIndex = HashMap<StopId, HashMap<usize, Leg>>;

#[derive(Debug)]
pub struct RaptorState {
    pub k: usize,
    k_arrivals: Vec<Vec<DateTime<Utc>>>,
    best_arrivals: Vec<DateTime<Utc>>,
    pub connection_index: ConnectionIndex,
}

impl RaptorState {
    pub fn init(num_stops: usize, start: StopId, departure: DateTime<Utc>) -> Self {
        let initial_taus = (0..num_stops)
            .map(|idx|
                if start.0 as usize != idx {
                    // Set initial earliest arrivals to "infinity"
                    DateTime::<Utc>::MAX_UTC
                } else {
                    // Set the departure node to instant departure
                    departure
                }
            )
            .collect::<Vec<DateTime<Utc>>>();

        // Table of earliest arrivals for each stop id (index corresponds to index in stops Vector)
        // called \tau_k (p) in the RAPTOR paper
        let stop_taus: Vec<Vec<DateTime<Utc>>> = vec![initial_taus.clone()];

        // called \tau^* in the RAPTOR paper (see section on local pruning)
        let best_arrivals = vec![DateTime::<Utc>::MAX_UTC; num_stops];

        Self {
            k: 0,
            k_arrivals: stop_taus,
            best_arrivals,
            connection_index: HashMap::new(),
        }
    }

    pub fn new_round(&mut self) {
        self.k += 1;

        // Set earliest arrival time with the current num_legs to the same value as for previous
        // number of legs (so where it was num_legs - 1).
        // This acts as an upper bound for the arrival time.
        self.k_arrivals.push(self.k_arrivals.last().unwrap().clone());
    }

    pub fn tau(&self, stop: &StopId) -> Option<&DateTime<Utc>> {
        self.k_arrivals.get(self.k)?.get(stop.0 as usize)
    }

    pub fn previous_tau(&self, stop: &StopId) -> Option<&DateTime<Utc>> {
        self.k_arrivals.get(self.k - 1)?.get(stop.0 as usize)
    }

    pub fn best_arrival(&self, stop: &StopId) -> Option<&DateTime<Utc>> {
        self.best_arrivals.get(stop.0 as usize)
    }

    pub fn set_ride(
        &mut self,
        start: StopId,
        end: StopId,
        departure: DateTime<Utc>,
        new_arrival: DateTime<Utc>,
        trip: TripId,
    ) {
        let end_idx = end.0 as usize;
        let prev_tau = self.k_arrivals[self.k][end_idx].clone();

        // τₖ(pᵢ) ← τₐᵣᵣ(t, pᵢ)
        self.k_arrivals[self.k][end_idx] = new_arrival;
        // τ*(pᵢ) ← τₐᵣᵣ(t, pᵢ)
        self.best_arrivals[end_idx] = min(new_arrival, prev_tau);

        self.connection_index
            .entry(end).or_insert(HashMap::new())
            .insert(self.k, Leg::Ride { trip, start, end, departure, arrival: new_arrival });
    }

    pub fn set_transfer(
        &mut self,
        start: StopId,
        end: StopId,
        duration: Duration,
    ) {
        let start_idx = start.0 as usize;
        let end_idx = end.0 as usize;
        let time_after_transfer = self.k_arrivals[self.k][start_idx] + duration;
        self.k_arrivals[self.k][end_idx] = time_after_transfer;
        self.best_arrivals[end_idx] = time_after_transfer;

        self.connection_index
            .entry(end).or_insert(HashMap::new())
            .insert(self.k, Leg::Transfer { start, end, duration });
    }

    pub fn backtrace(&self, target: StopId, departure: DateTime<Utc>) -> QueryResult<Journey> {
        let mut journeys: Vec<Journey> = vec![];

        for k in self.connection_index.get(&target).ok_or(NoRouteFound)?.keys() {
            if let Some(journey) = self.extract_journey(*k, target, departure) {
                journeys.push(journey);
            }
        }

        println!("journeys: {journeys:?}");

        // TODO: Return pareto-set of k's versus duration        
        // Determine the fastest route by calculating the final arrival time at the destination
        let fastest_journey = journeys.into_iter()
            .min_by_key(|journey| journey.arrival_when_starting_at(departure));
        
        fastest_journey.ok_or(NoRouteFound)
    }

    fn extract_journey(&self, k: usize, target: StopId, departure: DateTime<Utc>) -> Option<Journey> {
        let mut legs: Vec<Leg> = vec![];

        // collect the legs for this journey
        // iterate from the final target destination towards the start
        let mut curr_dest = target;
        let mut k = k;
        // time will be used to figure out whether a journey is actually feasible
        let mut time = None;
        
        while let Some(Some(leg)) = self.connection_index.get(&curr_dest).map(|x| x.get(&k)) {
            match leg {
                Leg::Ride { arrival, departure, .. } => {
                    // Only decrement k if the leg was a ride, since RAPTOR's rounds don't count
                    // transfers, only rides
                    k -= 1;

                    // If time is already initialized, we must check that this ride doesn't arrive too
                    // late for us to catch it. In this case, discard calculating the journey.
                    if let Some(time) = time {
                        if *arrival > time {
                            return None;
                        }
                    }
                    
                    // Update the time with the next fixed-time departure
                    time = Some(*departure);
                },
                Leg::Transfer { duration, .. } => {
                    // Do not decrement k, since RAPTOR's round don't count transfers
                    
                    // If there already has been a fixed-time transfer (aka a ride), update the time
                    if let Some(fixed_time) = time {
                        time = Some(fixed_time - *duration);
                    }
                }
            }

            curr_dest = *leg.start();
            legs.push(leg.clone());
        }

        legs.reverse();

        Some(Journey::from(legs))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_init() {
        let departure = DateTime::from_str("2042-06-24T12:00:00Z").unwrap();
        let mut res = RaptorState::init(4, StopId(2), departure);

        assert_eq!(res.k, 0);
        res.new_round();
        assert_eq!(res.k, 1);

        let initial_expected = vec![DateTime::<Utc>::MAX_UTC, DateTime::<Utc>::MAX_UTC, departure, DateTime::<Utc>::MAX_UTC];
        assert_eq!(res.k_arrivals, vec![
            initial_expected.clone(),
            initial_expected,
        ]);

        assert_eq!(
            res.best_arrivals,
            vec![DateTime::<Utc>::MAX_UTC, DateTime::<Utc>::MAX_UTC, departure, DateTime::<Utc>::MAX_UTC]
        );

        assert_eq!(res.connection_index, HashMap::new());
    }

    #[test]
    fn test_new_round() {
        let departure = DateTime::from_str("2042-06-24T12:00:00Z").unwrap();
        let mut state = RaptorState::init(2, StopId(0), departure);
        
        assert_eq!(
            state.tau(&StopId(0)),
            Some(&departure)
        );
        assert_eq!(
            state.tau(&StopId(1)),
            Some(&DateTime::<Utc>::MAX_UTC)
        );

        state.new_round();

        println!("{:?}", state);
        assert_eq!(
            state.tau(&StopId(0)),
            state.previous_tau(&StopId(0))
        );
        assert_eq!(
            state.tau(&StopId(1)),
            state.previous_tau(&StopId(1))
        );
    }
}