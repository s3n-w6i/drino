use serde_with::DisplayFromStr;
use crate::algorithms::queries::QueryType;
use common::types::StopId;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use serde_with::serde_as;

pub trait TargetCardinality<QT: QueryType>: Sized {
    type Output: Serialize;
}

#[serde_as]
#[derive(Deserialize)]
pub struct Single {
    #[serde_as(as = "DisplayFromStr")]
    pub target: StopId,
}

#[derive(Deserialize)]
pub struct Multiple<'a> {
    pub targets: Cow<'a, Vec<StopId>>,
}

#[derive(Deserialize)]
pub struct All;
