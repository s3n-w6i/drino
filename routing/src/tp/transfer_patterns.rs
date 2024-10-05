use crate::algorithm::RangeOutput;
use common::types::StopId;
use petgraph::prelude::GraphMap;
use petgraph::Directed;
use polars::error::PolarsResult;

/// https://ad.informatik.uni-freiburg.de/files/transferpatterns.pdf

#[derive(Debug)]
pub struct TransferPatterns(GraphMap<StopId, (), Directed>);

impl TransferPatterns {

    pub(crate) fn new() -> PolarsResult<Self> {
        Ok(Self(GraphMap::new()))
    }

    pub(crate) fn add_multiple(&mut self, results: Vec<RangeOutput>) -> PolarsResult<()> {
        let graph = &mut self.0;
        
        let all_journeys = results.into_iter()
            .map(|res| { res.journeys })
            .flatten();
        
        for journey in all_journeys {            
            for leg in journey.legs {
                graph.add_edge(*leg.start(), *leg.end(), ());
            }
        }

        Ok(())
    }
}