use itertools::Itertools;
use crate::algorithm::RangeOutput;
use common::types::StopId;
use petgraph::prelude::GraphMap;
use petgraph::Directed;
use polars::datatypes::DataType;
use polars::error::{PolarsError, PolarsResult};
use polars::frame::DataFrame;
use polars::frame::row::Row;
use polars::series::Series;

/// https://ad.informatik.uni-freiburg.de/files/transferpatterns.pdf

#[derive(Debug)]
pub struct TransferPatternsGraph(GraphMap<StopId, (), Directed>);

impl TransferPatternsGraph {

    pub(crate) fn new() -> PolarsResult<Self> {
        Ok(Self(GraphMap::new()))
    }

    pub(crate) fn add(&mut self, results: Vec<RangeOutput>) -> PolarsResult<()> {
        let graph = &mut self.0;
        
        let all_journeys = results.into_iter()
            .map(|res| { res.journeys })
            .flatten();
        
        for journey in all_journeys {
            for leg in journey.legs {
                // Add an edge from the end to start. This is reversed, so that on query time, we
                // can start at our target station and then find a way to the origin station
                // efficiently.
                // GraphMap's add_edge function inserts missing nodes automatically.
                graph.add_edge(*leg.end(), *leg.start(), ());
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct TransferPatternsTable(pub(crate) DataFrame);

impl TransferPatternsTable {
    pub(crate) fn new() -> PolarsResult<Self> {
        let start_series = Series::new_empty("start".into(), &DataType::UInt32);
        let end_series = Series::new_empty("end".into(), &DataType::UInt32);

        Ok(Self(DataFrame::new(vec![start_series, end_series])?))
    }
}

impl TryFrom<TransferPatternsGraph> for TransferPatternsTable {
    type Error = PolarsError;

    fn try_from(TransferPatternsGraph(graph): TransferPatternsGraph) -> Result<Self, Self::Error> {
        let mut table = Self::new()?;
        
        let rows = graph.all_edges()
            .map(|(a, b, ..)| {
                Row::new(vec![a.into(), b.into()])
            })
            .collect_vec();
        
        if !rows.is_empty() {
            table.0.vstack_mut_unchecked(
                &DataFrame::from_rows(&rows)?
            );

            table.0.align_chunks();
        }
        
        Ok(table)
    }
}