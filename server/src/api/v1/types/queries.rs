use routing::algorithm::{Single, Multiple, EarliestArrivalOutput};
use serde::{Deserialize, Serialize};
use common::types::StopId;

/*
#[derive(Deserialize)]
#[serde(remote = "StopId")]
struct StopIdDef(u32);

#[derive(Deserialize)]
#[serde(remote = "Single")]
pub struct SingleDef {
    #[serde(with = "StopIdDef")]
    target: StopId
}

#[derive(Deserialize)]
#[serde(remote = "Multiple")]
pub struct MultipleDef<'lifetime> {
    #[serde(with = "StopIdDef")]
    targets: &'lifetime Vec<StopId>
}

#[derive(Serialize)]
#[serde(remote = "EarliestArrivalOutput")]
struct EarliestArrivalOutputDef {
    journey: Journey
}
*/