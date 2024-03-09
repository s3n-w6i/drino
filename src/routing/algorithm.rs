use async_trait::async_trait;
use polars::prelude::LazyFrame;

#[async_trait]
pub trait RoutingAlgorithm {
    async fn preprocess(input: PreprocessingInput) -> PreprocessingResult;
    async fn query();
}

pub struct PreprocessingInput {
    pub services: LazyFrame, // corresponds to calendar.txt in GTFS
    pub stops: LazyFrame,
    pub trips: LazyFrame,
    pub stop_times: LazyFrame,
}
pub type PreprocessingResult = Result<(), ()>;