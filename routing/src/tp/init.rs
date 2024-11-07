use crate::algorithm::{AllRange, PreprocessInit, PreprocessingInput, PreprocessingResult, Range};
use crate::direct_connections::DirectConnections;
use crate::raptor::RaptorAlgorithm;
use crate::tp::transfer_patterns::TransferPatternsGraph;
use crate::tp::TransferPatternsAlgorithm;
use async_trait::async_trait;
use chrono::{DateTime, Duration};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::sync::{Arc, Mutex};

// TODO: Experiment with this value to see which one is useful
const CHUNK_SIZE: u64 = 5;

#[async_trait]
impl PreprocessInit for TransferPatternsAlgorithm {
    fn preprocess(input: PreprocessingInput, progress_bars: Option<&MultiProgress>) -> PreprocessingResult<Self> {
        let direct_connections = DirectConnections::try_from(input.clone())?;
        let raptor = Arc::new(RaptorAlgorithm::preprocess(input, direct_connections.clone())?);
        let tp_graph = Arc::new(Mutex::new(TransferPatternsGraph::new()?));

        let pb = progress_bars.map(|pbs| {
            pbs.add(
                ProgressBar::new(raptor.stops.len() as u64)
                    .with_message("Processing stops in cluster...")
                    .with_style(
                        ProgressStyle::with_template("[{elapsed}] {msg} [{wide_bar}] {human_pos}/{human_len}")
                            .unwrap().progress_chars("=> ")
                    )
            )
        });

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
                    tp_graph.add(vec![range_out]).unwrap();
                }

                pb.clone().map(|pb| pb.inc(1));
            });

        pb.map(|pb| { pb.finish_with_message("All stops in cluster finished") });

        let tp_graph = Arc::try_unwrap(tp_graph)
            .expect("Lock is still owned by others").into_inner().unwrap();
        
        dbg!(&tp_graph.node_count());

        Ok(Self {
            direct_connections,
            transfer_patterns: tp_graph,
        })
    }
}