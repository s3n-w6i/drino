use crate::algorithm::{AllRange, PreprocessInit, PreprocessingInput, PreprocessingResult, Range};
use crate::direct_connections::DirectConnections;
use crate::raptor::RaptorAlgorithm;
use crate::tp::transfer_patterns::TransferPatternsGraphs;
use crate::tp::TransferPatternsAlgorithm;
use async_trait::async_trait;
use chrono::{DateTime, Duration};
use common::util::logging::run_with_pb;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::sync::{Arc, Mutex};

#[async_trait]
impl PreprocessInit for TransferPatternsAlgorithm {
    fn preprocess(input: PreprocessingInput) -> PreprocessingResult<Self> {
        let direct_connections = DirectConnections::try_from(input.clone())?;
        let raptor = Arc::new(RaptorAlgorithm::preprocess(input.clone(), direct_connections.clone())?);
        let tp_graph = Arc::new(Mutex::new(
            TransferPatternsGraphs::new(raptor.stops.len())
        ));

        let total = raptor.stops.len() as u64;
        run_with_pb("preprocessing", "Calculating local transfers in a single cluster", total, false, |pb| {
            raptor.stops.par_iter()
                .for_each(|stop| {
                    let raptor = Arc::clone(&raptor);
                    let tp_graph = Arc::clone(&tp_graph);

                    let result = raptor.query_range_all(
                        Range {
                            earliest_departure: DateTime::from_timestamp_millis(0).unwrap(),
                            start: *stop,
                            range: Duration::weeks(1),
                        }
                    );

                    if let Ok(range_out) = result {
                        // Add this chunk to our existing transfer patterns
                        let mut tp_graph = tp_graph.lock().unwrap();
                        tp_graph.add(vec![range_out]);
                    }

                    pb.inc(1);
                });
        });

        let tp_graph = Arc::try_unwrap(tp_graph)
            .expect("Lock is still owned by others").into_inner().unwrap();

        #[cfg(debug_assertions)] {
            // Check that graphs are acyclic. Expensive to compute, so only do that in debug.
            tp_graph.validate();
        }

        tp_graph.print(0.into());

        Ok(Self {
            direct_connections,
            transfer_patterns: tp_graph,
        })
    }
}