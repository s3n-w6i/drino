use chrono::{DateTime, Duration, Utc};
use crate::raptor::RaptorAlgorithm;
use crate::transfers::CrowFlyTransferProvider;
use common::types::{LineId, SeqNum, StopId, TripId};
use geo::{Coord, HaversineDistance, Point};
use hashbrown::{HashMap, HashSet};
use common::util::speed::MAX_WALKING_SPEED;

pub(crate) const STOP3_COORD: Coord<f32> = Coord { x: 0.0, y: 0.0 };
pub(crate) const STOP4_COORD: Coord<f32> = Coord { x: 0.001, y: 0.01 };
pub(crate) fn duration_stop3_stop4() -> Duration {
    MAX_WALKING_SPEED.time_to_travel_distance(
        Point::from(STOP3_COORD).haversine_distance(&Point::from(STOP4_COORD))
    )
}

/// Test case 4 has some specialties:
/// - Stop 3 and 4 are quite close together, so walking between them is feasible
/// - Line 101 travel "back", and usually doesn't contribute to reaching a target
pub(crate) fn generate_case_4() -> RaptorAlgorithm {
    let dep20 = DateTime::<Utc>::from_timestamp(20, 0).unwrap();
    let dep220 = DateTime::<Utc>::from_timestamp(220, 0).unwrap();
    let dep110 = DateTime::<Utc>::from_timestamp(110, 0).unwrap();
    let dep310 = DateTime::<Utc>::from_timestamp(310, 0).unwrap();
    let dep0 = DateTime::<Utc>::from_timestamp(0, 0).unwrap();
    let dep400 = DateTime::<Utc>::from_timestamp(400, 0).unwrap();
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

    RaptorAlgorithm {
        stops: vec![0, 1, 2, 3, 4].into_iter().map(|x| StopId(x)).collect(),
        stops_by_line: HashMap::from([
            // Line 100: 0 --> 2 --> 3
            (LineId(100), vec![StopId(0), StopId(2), StopId(3)]),
            // Line 101: 1 <-- 2 <-- 3
            (LineId(101), vec![StopId(3), StopId(2), StopId(1)]),
            // Line 120: 1 --> 2 --> 4
            (LineId(120), vec![StopId(1), StopId(2), StopId(4)]),
            // Line 130 ("express line"): 0 --> 3
            (LineId(130), vec![StopId(0), StopId(3)]),
        ]),
        lines_by_stops: HashMap::from([
            (StopId(0), HashSet::from([(LineId(100), SeqNum(0)), (LineId(130), SeqNum(0))])),
            (StopId(1), HashSet::from([(LineId(101), SeqNum(2)), (LineId(120), SeqNum(0))])),
            (StopId(2), HashSet::from([(LineId(100), SeqNum(1)), (LineId(101), SeqNum(1)), (LineId(120), SeqNum(1))])),
            (StopId(3), HashSet::from([(LineId(100), SeqNum(2)), (LineId(101), SeqNum(0)), (LineId(130), SeqNum(1))])),
            (StopId(4), HashSet::from([(LineId(120), SeqNum(2))])),
        ]),
        departures: HashMap::from([
            // Line 100
            ((TripId(100_1), StopId(0)), dep20),
            ((TripId(100_1), StopId(2)), dep110),
            ((TripId(100_2), StopId(0)), dep220),
            ((TripId(100_2), StopId(2)), dep310),
            // Line 101
            ((TripId(101_1), StopId(3)), dep20),
            ((TripId(101_1), StopId(2)), dep110),
            ((TripId(101_2), StopId(3)), dep220),
            ((TripId(101_2), StopId(2)), dep310),
            // Line 120
            ((TripId(120_1), StopId(1)), dep0),
            ((TripId(120_1), StopId(2)), dep150),
            ((TripId(120_2), StopId(1)), dep400),
            ((TripId(120_2), StopId(2)), dep550),
            // Line 130
            ((TripId(130_1), StopId(0)), dep0),
        ]),
        arrivals: HashMap::from([
            // Line 100
            ((TripId(100_1), StopId(2)), arr100),
            ((TripId(100_1), StopId(3)), arr300),
            ((TripId(100_2), StopId(2)), arr150),
            ((TripId(100_2), StopId(3)), arr350),
            // Line 101
            ((TripId(101_1), StopId(2)), arr100),
            ((TripId(101_1), StopId(1)), arr150),
            ((TripId(101_2), StopId(2)), arr300),
            ((TripId(101_2), StopId(1)), arr350),
            // Line 120
            ((TripId(120_1), StopId(2)), arr200),
            ((TripId(120_1), StopId(4)), arr300),
            ((TripId(120_2), StopId(2)), arr600),
            ((TripId(120_2), StopId(4)), arr700),
            // Line 130
            ((TripId(130_1), StopId(3)), arr250),
        ]),
        trips_by_line_and_stop: HashMap::from([
            ((LineId(100), StopId(0)), vec![(dep20, TripId(100_1)), (dep220, TripId(100_2))]),
            ((LineId(100), StopId(2)), vec![(dep110, TripId(100_1)), (dep310, TripId(100_2))]),
            ((LineId(101), StopId(3)), vec![(dep20, TripId(101_1)), (dep220, TripId(101_2))]),
            ((LineId(101), StopId(2)), vec![(dep110, TripId(101_1)), (dep310, TripId(101_2))]),
            ((LineId(120), StopId(1)), vec![(dep0, TripId(120_1)), (dep400, TripId(120_2))]),
            ((LineId(120), StopId(2)), vec![(dep150, TripId(120_1)), (dep550, TripId(120_2))]),
            ((LineId(130), StopId(0)), vec![(dep0, TripId(130_1))]),
        ]),
        transfer_provider: CrowFlyTransferProvider::from(vec![
            Coord { x: -10.0, y: -10.0 },
            Coord { x: 42.0, y: 0.0 },
            Coord { x: -25.0, y: 10.0 },
            STOP3_COORD,
            STOP4_COORD,
        ]),
    }
}

#[macro_export]
macro_rules! earliest_arrival_tests {
    ($t: ty) => {
        use chrono::{DateTime, Utc};
        use geo::{HaversineDistance, Point};
        
        use crate::algorithm::{ EarliestArrival, Journey, Leg, Single, SingleEarliestArrival};
        use common::types::{LineId, SeqNum, StopId, TripId};
        use common::util::speed::MAX_WALKING_SPEED;
        use crate::tests::{STOP3_COORD, STOP4_COORD};
        
        ///  0 ---Ride--> 1
        #[tokio::test]
        async fn test_query_earliest_1() {
            // todo: let algorithm = <$t>::preprocess(todo!(), todo!()).unwrap();
            let raptor = RaptorAlgorithm {
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
                transfer_provider: CrowFlyTransferProvider::from(vec![
                    Coord { x: 0.0, y: 0.0 },
                    Coord { x: 40.0, y: 0.0 },
                ]),
            };

            let res = raptor.query_ea(
                EarliestArrival { start: StopId(0), departure: DateTime::UNIX_EPOCH },
                Single { target: StopId(1) },
            ).await.unwrap();

            assert_eq!(res.journey, Journey {
                legs: vec![Leg::Ride {
                    start: StopId(0),
                    end: StopId(1),
                    departure: DateTime::<Utc>::from_timestamp(100, 0).unwrap(),
                    arrival: DateTime::<Utc>::from_timestamp(500, 0).unwrap(),
                    trip: TripId(0),
                }]
            });

            // query a little later (missed the only connection there is)
            let res = raptor.query_ea(
                EarliestArrival { start: StopId(0), departure: DateTime::<Utc>::from_timestamp(300, 0).unwrap() },
                Single { target: StopId(1) },
            ).await;

            assert!(res.is_err());
        }

        ///   0 ---Ride--> 1 ---Ride--> 2
        #[tokio::test]
        async fn test_query_earliest_2() {
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

            let res = raptor.query_ea(
                EarliestArrival { start: StopId(0), departure: DateTime::UNIX_EPOCH },
                Single { target: StopId(2) },
            ).await.unwrap();

            assert_eq!(res.journey, Journey {
                legs: vec![
                    Leg::Ride {
                        start: StopId(0),
                        end: StopId(1),
                        departure: DateTime::<Utc>::from_timestamp(100, 0).unwrap(),
                        arrival: DateTime::<Utc>::from_timestamp(500, 0).unwrap(),
                        trip: TripId(0),
                    },
                    Leg::Ride {
                        start: StopId(1),
                        end: StopId(2),
                        departure: DateTime::<Utc>::from_timestamp(1000, 0).unwrap(),
                        arrival: DateTime::<Utc>::from_timestamp(1500, 0).unwrap(),
                        trip: TripId(1),
                    },
                ]
            });
        }

        ///   0 ---Ride--> 1 ---Transfer--> 2 ---Ride--> 3
        #[tokio::test]
        async fn test_query_earliest_3() {
            let coord_1 = Coord { x: 40.00, y: 0.0 };
            let coord_2 = Coord { x: 40.01, y: 0.0 };
            let raptor = RaptorAlgorithm {
                stops: vec![0, 1, 2, 3].into_iter().map(|x| StopId(x)).collect(),
                stops_by_line: HashMap::from([
                    (LineId(0), vec![StopId(0), StopId(1)]),
                    (LineId(1), vec![StopId(2), StopId(3)]),
                ]),
                lines_by_stops: HashMap::from([
                    (StopId(0), HashSet::from([(LineId(0), SeqNum(0))])),
                    (StopId(1), HashSet::from([(LineId(0), SeqNum(1))])),
                    (StopId(2), HashSet::from([(LineId(1), SeqNum(0))])),
                    (StopId(3), HashSet::from([(LineId(1), SeqNum(1))])),
                ]),
                departures: HashMap::from([
                    ((TripId(0), StopId(0)), DateTime::<Utc>::from_timestamp(100, 0).unwrap()),
                    ((TripId(1), StopId(2)), DateTime::<Utc>::from_timestamp(1000, 0).unwrap()),
                ]),
                arrivals: HashMap::from([
                    ((TripId(0), StopId(1)), DateTime::<Utc>::from_timestamp(500, 0).unwrap()),
                    ((TripId(1), StopId(3)), DateTime::<Utc>::from_timestamp(1500, 0).unwrap()),
                ]),
                trips_by_line_and_stop: HashMap::from([
                    ((LineId(0), StopId(0)), vec![(DateTime::<Utc>::from_timestamp(100, 0).unwrap(), TripId(0))]),
                    ((LineId(1), StopId(2)), vec![(DateTime::<Utc>::from_timestamp(1000, 0).unwrap(), TripId(1))]),
                ]),
                transfer_provider: CrowFlyTransferProvider::from(vec![
                    Coord { x: 0.0, y: 0.0 },
                    coord_1,
                    coord_2,
                    Coord { x: -40.0, y: 0.0 },
                ]),
            };

            let res = raptor.query_ea(
                EarliestArrival { start: StopId(0), departure: DateTime::UNIX_EPOCH },
                Single { target: StopId(3) },
            ).await.unwrap();

            assert_eq!(res.journey, Journey {
                legs: vec![
                    Leg::Ride {
                        start: StopId(0),
                        end: StopId(1),
                        departure: DateTime::<Utc>::from_timestamp(100, 0).unwrap(),
                        arrival: DateTime::<Utc>::from_timestamp(500, 0).unwrap(),
                        trip: TripId(0),
                    },
                    Leg::Transfer {
                        start: StopId(1),
                        end: StopId(2),
                        duration: MAX_WALKING_SPEED.time_to_travel_distance(
                            Point::from(coord_1).haversine_distance(&Point::from(coord_2))
                        ),
                    },
                    Leg::Ride {
                        start: StopId(2),
                        end: StopId(3),
                        departure: DateTime::<Utc>::from_timestamp(1000, 0).unwrap(),
                        arrival: DateTime::<Utc>::from_timestamp(1500, 0).unwrap(),
                        trip: TripId(1),
                    },
                ]
            });
        }

        #[tokio::test]
        async fn test_query_earliest_4() {
            let dep20 = DateTime::<Utc>::from_timestamp(20, 0).unwrap();
            let dep220 = DateTime::<Utc>::from_timestamp(220, 0).unwrap();
            let dep110 = DateTime::<Utc>::from_timestamp(110, 0).unwrap();
            let dep310 = DateTime::<Utc>::from_timestamp(310, 0).unwrap();
            let dep0 = DateTime::<Utc>::from_timestamp(0, 0).unwrap();
            let dep400 = DateTime::<Utc>::from_timestamp(400, 0).unwrap();
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
            // Takes around ~652s
            let res = raptor.query_ea(
                EarliestArrival { start: StopId(0), departure: dep0 },
                Single { target: StopId(4) },
            ).await.unwrap();

            assert_eq!(
                res.journey,
                Journey { legs: vec![
                    Leg::Ride {
                        trip: TripId(130_1),
                        start: StopId(0),
                        end: StopId(3),
                        departure: dep0,
                        arrival: arr250,
                    },
                    Leg::Transfer {
                        start: StopId(3),
                        end: StopId(4),
                        duration: duration_stop3_stop4(),
                    },
                ]}
            );

            // 0@20s   ---Ride(100_1)-->   2@100s, 2@150s   ---Ride(120_1)-->   4@300s
            // this connection arrives well before the following, which would take ~702s instead of 300s:
            // 0@20s   ---Ride(100_1)-->   3@300s   ---Transfer-->   4@702s
            let res = raptor.query_ea(
                EarliestArrival { start: StopId(0), departure: DateTime::<Utc>::from_timestamp(1, 0).unwrap() },
                Single { target: StopId(4) },
            ).await.unwrap();

            assert_eq!(
                res.journey,
                Journey { legs: vec![
                    Leg::Ride {
                        trip: TripId(100_1),
                        start: StopId(0), end: StopId(2),
                        departure: dep20, arrival: arr100,
                    },
                    Leg::Ride {
                        trip: TripId(120_1),
                        start: StopId(2), end: StopId(4),
                        departure: dep150, arrival: arr300,
                    },
                ]}
            );
            
            // TODO: More cases
        }
    };
}