use crate::algorithms::errors::QueryResult;
use crate::algorithms::queries::cardinality::TargetCardinality;
use crate::algorithms::RoutingAlgorithm;
use common::types::StopId;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer};

pub mod cardinality;
pub mod earliest_arrival;
pub mod latest_departure;
pub mod range;

pub trait Queryable<QT: QueryType, TC: TargetCardinality<QT>>: RoutingAlgorithm {
    fn query(&self, input: QT::Input, target_cardinality: TC) -> QueryResult<TC::Output>;
}

pub trait QueryType: Sized {
    type Input: DeserializeOwned;
    // Output type is defined in cardinality impl
}

#[derive(Deserialize)]
pub struct SerDeQuery<QT: QueryType> {
    input: QT::Input,
    targets: Vec<StopId>,
}

pub struct Query<QT: QueryType, TC: TargetCardinality<QT>> {
    input: QT::Input,
    target_cardinality: TC,
}

impl<'de, QT, TC> Deserialize<'de> for Query<QT, TC>
where
    QT: QueryType,
    TC: TargetCardinality<QT>,
    QT::Input: Deserialize<'de>,
    TC: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct QueryHelper<Input, TC> {
            input: Input,
            target_cardinality: TC,
        }

        let helper = QueryHelper::<QT::Input, TC>::deserialize(deserializer)?;
        Ok(Self {
            input: helper.input,
            target_cardinality: helper.target_cardinality,
        })
    }
}

// Utility function to run a generic query on an algorithm
pub fn run<A, QT, TC>(algorithm: &A, query: Query<QT, TC>) -> QueryResult<TC::Output>
where
    A: Queryable<QT, TC>,
    QT: QueryType,
    TC: TargetCardinality<QT>,
{
    algorithm.query(query.input, query.target_cardinality)
}
