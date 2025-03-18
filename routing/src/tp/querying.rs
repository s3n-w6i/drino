use crate::algorithms::errors::QueryResult;
use crate::algorithms::queries::earliest_arrival::{
    EarliestArrival, EarliestArrivalInput, EarliestArrivalOutput,
};
use crate::algorithms::queries::{cardinality, Queryable};
use crate::tp::TransferPatternsAlgorithm;

impl Queryable<EarliestArrival, cardinality::Single> for TransferPatternsAlgorithm {
    fn query(
        &self,
        input: EarliestArrivalInput,
        cardinality: cardinality::Single,
    ) -> QueryResult<EarliestArrivalOutput> {
        drop(input);
        drop(cardinality);
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::journey::{Journey, Leg};
    use crate::tests::*;
    use crate::tp::TransferPatternsAlgorithm;
    use chrono::DateTime;
    use common::types::{StopId, TripId};
    use crate::algorithms::initialization::ByPreprocessing;
    use crate::algorithms::queries::earliest_arrival::{EarliestArrivalInput, EarliestArrivalOutput};
    use crate::algorithms::queries::{cardinality, Queryable};

    #[test]
    fn single_ea_case_1() {
        let case_1_input = case_1::generate_preprocessing_input().unwrap();
        let alg = TransferPatternsAlgorithm::preprocess(case_1_input, false).unwrap();

        let actual = alg
            .query(
                EarliestArrivalInput {
                    earliest_departure: DateTime::UNIX_EPOCH,
                    start: StopId(0),
                },
                cardinality::Single { target: StopId(1) },
            )
            .unwrap();
        let expected = EarliestArrivalOutput {
            journey: Journey::from(vec![Leg::Ride {
                trip: TripId(0),
                boarding_stop: StopId(0),
                alight_stop: StopId(1),
                boarding_time: DateTime::from_timestamp_millis(100).unwrap(),
                alight_time: DateTime::from_timestamp_millis(500).unwrap(),
            }]),
        };
        assert_eq!(expected, actual);
    }
}
