use crate::algorithms::queries::QueryType;
use common::types::StopId;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

pub trait TargetCardinality<QT: QueryType>: Sized {
    type Output: Serialize;
}

#[derive(Deserialize)]
pub struct Single {
    pub target: StopId,
}

#[derive(Deserialize)]
pub struct Multiple<'a> {
    pub targets: Cow<'a, Vec<StopId>>,
}

#[derive(Deserialize)]
pub struct All;
