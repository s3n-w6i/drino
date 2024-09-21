use crate::algorithm::{AllRange, PreprocessInit, PreprocessingInput, PreprocessingResult, Range};
use crate::direct_connections::DirectConnections;
use crate::raptor::RaptorAlgorithm;
use crate::tp::transfer_patterns::{TransferPatternMap, TransferPatterns};
use crate::tp::TransferPatternsAlgorithm;
use async_trait::async_trait;
use chrono::{DateTime, Duration};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::sync::Arc;

#[async_trait]
impl PreprocessInit for TransferPatternsAlgorithm {
    fn preprocess(input: PreprocessingInput) -> PreprocessingResult<Self> {
        let direct_connections = DirectConnections::try_from(input.clone())?;
        let raptor = Arc::new(RaptorAlgorithm::preprocess(input, direct_connections.clone())?);

        &raptor.stops.par_iter()
            .for_each(|stop| {
                let raptor = Arc::clone(&raptor);
                let res  = &raptor.query_range_all(
                    Range {
                        earliest_departure: DateTime::from_timestamp_millis(0).unwrap(),
                        start: *stop,
                        range: Duration::weeks(1),
                    }
                );
            });

        /*let direct_connections = DirectConnections::try_from(input)?;
        let network = TransitNetwork::try_from(direct_connections.clone())?;
        debug!("Network graph stats: {} nodes ({} transfer), {} edges", network.graph.raw_nodes().len(), network.transfer_node_ids.len(), network.graph.raw_edges().len());
        let transfer_patterns = TransferPatterns::from(network);

        Ok(Self {
            direct_connections,
            transfer_patterns,
        })*/

        Ok(Self {
            direct_connections,
            transfer_patterns: TransferPatterns(TransferPatternMap::new()),
        })
    }
}