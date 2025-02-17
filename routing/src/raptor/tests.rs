use chrono::{DateTime, Duration, Utc};
use hashbrown::{HashMap, HashSet};
use ndarray::array;
use common::types::{LineId, SeqNum, StopId, TripId};
use common::util::duration::INFINITY;
use crate::raptor::{RaptorAlgorithm, StopMapping};
use crate::transfers::fixed_time::FixedTimeTransferProvider;

#[allow(clippy::inconsistent_digit_grouping)]
/// Test case 4 has some specialties:
/// - Stop 3 and 4 are quite close together, so walking between them is feasible
/// - Line 101 travel "back", and usually doesn't contribute to reaching a target
pub(crate) fn generate_case_4() -> RaptorAlgorithm {
    let dep20 = DateTime::<Utc>::from_timestamp(20, 0).unwrap();
    let dep220 = DateTime::<Utc>::from_timestamp(220, 0).unwrap();
    let dep90 = DateTime::<Utc>::from_timestamp(90, 0).unwrap();
    let dep110 = DateTime::<Utc>::from_timestamp(110, 0).unwrap();
    let dep310 = DateTime::<Utc>::from_timestamp(310, 0).unwrap();
    let dep0 = DateTime::<Utc>::from_timestamp(0, 0).unwrap();
    let dep400 = DateTime::<Utc>::from_timestamp(400, 0).unwrap();
    let dep490 = DateTime::<Utc>::from_timestamp(490, 0).unwrap();

    let arr80 = DateTime::<Utc>::from_timestamp(80, 0).unwrap();
    let arr480 = DateTime::<Utc>::from_timestamp(480, 0).unwrap();
    let arr100 = DateTime::<Utc>::from_timestamp(100, 0).unwrap();
    let arr300 = DateTime::<Utc>::from_timestamp(300, 0).unwrap();
    let arr150 = DateTime::<Utc>::from_timestamp(150, 0).unwrap();
    let arr350 = DateTime::<Utc>::from_timestamp(350, 0).unwrap();
    let arr700 = DateTime::<Utc>::from_timestamp(700, 0).unwrap();
    let arr250 = DateTime::<Utc>::from_timestamp(250, 0).unwrap();
    
    let duration_3_to_4 = Duration::seconds(410);

    RaptorAlgorithm {
        stop_mapping: StopMapping(vec![0, 1, 2, 3, 4].into_iter().map(StopId).collect()),
        stops_by_line: HashMap::from([
            // Line 100: 0 --> 2 --> 3
            (LineId(100), vec![(StopId(0), 0), (StopId(2), 0), (StopId(3), 0)]),
            // Line 101: 1 <-- 2 <-- 3
            (LineId(101), vec![(StopId(3), 0), (StopId(2), 0), (StopId(1), 0)]),
            // Line 120: 1 --> 2 --> 4
            (LineId(120), vec![(StopId(1), 0), (StopId(2), 0), (StopId(4), 0)]),
            // Line 130 ("express line"): 0 --> 3
            (LineId(130), vec![(StopId(0), 0), (StopId(3), 0)]),
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
            ((TripId(100_1), StopId(0), 0), dep20),
            ((TripId(100_1), StopId(2), 0), dep110),
            ((TripId(100_2), StopId(0), 0), dep220),
            ((TripId(100_2), StopId(2), 0), dep310),
            // Line 101
            ((TripId(101_1), StopId(3), 0), dep20),
            ((TripId(101_1), StopId(2), 0), dep110),
            ((TripId(101_2), StopId(3), 0), dep220),
            ((TripId(101_2), StopId(2), 0), dep310),
            // Line 120
            ((TripId(120_1), StopId(1), 0), dep0),
            ((TripId(120_1), StopId(2), 0), dep90),
            ((TripId(120_2), StopId(1), 0), dep400),
            ((TripId(120_2), StopId(2), 0), dep490),
            // Line 130
            ((TripId(130_1), StopId(0), 0), dep0),
        ]),
        arrivals: HashMap::from([
            // Line 100
            ((TripId(100_1), StopId(2), 0), arr100),
            ((TripId(100_1), StopId(3), 0), arr300),
            ((TripId(100_2), StopId(2), 0), arr250),
            ((TripId(100_2), StopId(3), 0), arr350),
            // Line 101
            ((TripId(101_1), StopId(2), 0), arr100),
            ((TripId(101_1), StopId(1), 0), arr150),
            ((TripId(101_2), StopId(2), 0), arr300),
            ((TripId(101_2), StopId(1), 0), arr350),
            // Line 120
            ((TripId(120_1), StopId(2), 0), arr80),
            ((TripId(120_1), StopId(4), 0), arr300),
            ((TripId(120_2), StopId(2), 0), arr480),
            ((TripId(120_2), StopId(4), 0), arr700),
            // Line 130
            ((TripId(130_1), StopId(3), 0), arr250),
        ]),
        trips_by_line_and_stop: HashMap::from([
            ((LineId(100), StopId(0)), vec![(dep20, TripId(100_1)), (dep220, TripId(100_2))]),
            ((LineId(100), StopId(2)), vec![(dep110, TripId(100_1)), (dep310, TripId(100_2))]),
            ((LineId(101), StopId(3)), vec![(dep20, TripId(101_1)), (dep220, TripId(101_2))]),
            ((LineId(101), StopId(2)), vec![(dep110, TripId(101_1)), (dep310, TripId(101_2))]),
            ((LineId(120), StopId(1)), vec![(dep0, TripId(120_1)), (dep400, TripId(120_2))]),
            ((LineId(120), StopId(2)), vec![(dep90, TripId(120_1)), (dep490, TripId(120_2))]),
            ((LineId(130), StopId(0)), vec![(dep0, TripId(130_1))]),
        ]),
        transfer_provider: Box::new(FixedTimeTransferProvider {
            duration_matrix: array![
                [Duration::zero(), INFINITY, INFINITY,  INFINITY, INFINITY],
                [INFINITY, Duration::zero(), INFINITY,  INFINITY, INFINITY],
                [INFINITY, INFINITY, Duration::zero(),  INFINITY, INFINITY],
                [INFINITY, INFINITY, INFINITY, Duration::zero(), duration_3_to_4  ],
                [INFINITY, INFINITY, INFINITY, duration_3_to_4,  Duration::zero()  ],
            ]
        })
    }
}