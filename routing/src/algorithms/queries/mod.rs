use crate::algorithms::errors::QueryResult;
use crate::algorithms::queries::cardinality::TargetCardinality;
use crate::algorithms::RoutingAlgorithm;
use serde::de::DeserializeOwned;

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
