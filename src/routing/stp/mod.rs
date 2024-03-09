mod preprocessing;

use async_trait::async_trait;
use crate::routing::algorithm::{PreprocessingInput, PreprocessingResult, RoutingAlgorithm};
use crate::routing::stp::preprocessing::cluster::cluster;

pub struct ScalableTransferPatternsAlgorithm;

#[async_trait]
impl RoutingAlgorithm for ScalableTransferPatternsAlgorithm {
    async fn preprocess(input: PreprocessingInput) -> PreprocessingResult {
        cluster(&input.services, &input.stops, &input.stop_times, &input.trips)
            .await.expect("Clustering failed");
        Ok(())
    }

    async fn query() {
        todo!()
    }
}