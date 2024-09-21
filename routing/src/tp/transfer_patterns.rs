use crate::algorithm::{Leg, RangeOutput};
use itertools::Itertools;
use polars::error::PolarsResult;
use polars::frame::row::Row;
use polars::frame::{DataFrame, UniqueKeepStrategy};
use polars::prelude::{AnyValue, DataType, NamedFrom};
use polars::series::Series;

/// https://ad.informatik.uni-freiburg.de/files/transferpatterns.pdf

pub type TransferPatternDf = DataFrame;

#[derive(Debug)]
pub struct TransferPatterns(TransferPatternDf);

impl TransferPatterns {
    
    pub(crate) fn new() -> PolarsResult<Self> {
        let start_series = Series::new_empty("start".into(), &DataType::UInt32);
        let end_series = Series::new_empty("end".into(), &DataType::UInt32);
        let pattern_series = Series::new_empty(
            "pattern".into(),
            // List of individual "leg-templates" (aka legs of a transfer pattern).
            // Contains the leg's line ID.
            // If a leg's value is None, then treat it as a transfer step (walking, cycling etc.).
            &DataType::List(Box::new(DataType::UInt32))
        );
        
        Ok(Self(DataFrame::new(vec![start_series, end_series, pattern_series])?))
    }
    
    pub(crate) fn add_multiple(&mut self, results: Vec<RangeOutput>) -> PolarsResult<()> {
        dbg!(&results.len());
        
        let all_journeys = results.into_iter()
            .map(|res| { res.journeys })
            .flatten();
        
        let rows = all_journeys
            .map(|journey| {
                let start = journey.start().clone();
                let end = journey.end().clone();
                // We are only interested in the lines, not the trips themselves
                let generic_leg_line_ids = journey.legs.into_iter()
                    .map(|leg| {
                        match leg {
                            Leg::Ride { trip, .. } => Some(trip), // TODO: Use line id!
                            Leg::Transfer { .. } => None
                        }
                    })
                    .map(|id| {
                        match id {
                            None => AnyValue::Null,
                            Some(trip_id) => trip_id.into()
                        }
                    })
                    .collect_vec();

                (start, end, generic_leg_line_ids)
            })
            .unique()
            .map(|(start, end, pattern)| {
                let pattern_list = AnyValue::List(Series::new("pattern".into(), pattern));
                Row::new(vec![start.into(), end.into(), pattern_list])
            })
            .collect_vec();
        
        if rows.len() > 0 {
            // Append new rows to existing 
            self.0.vstack_mut_unchecked(
                &DataFrame::from_rows(&rows)?
            );
        }
        
        Ok(())
    }
    
    pub(crate) fn align_chunks(&mut self) {
        self.0.align_chunks();
    }

    pub(crate) fn remove_duplicates(&mut self) -> PolarsResult<()> {
        self.0 = self.0.unique_impl(false, None, UniqueKeepStrategy::Any, None)?;
        Ok(())
    }
}