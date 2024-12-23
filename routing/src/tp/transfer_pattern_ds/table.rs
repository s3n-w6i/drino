use crate::algorithm::{PreprocessingResult, RangeOutput};
use crate::journey::Journey;
use polars::datatypes::DataType;
use polars::error::{PolarsError, PolarsResult};
use polars::frame::row::Row;
use polars::frame::{DataFrame, UniqueKeepStrategy};
use polars::prelude::{AnyValue, Column};
use polars::series::Series;

/// columns:
/// - "start" (stop id)
/// - "target" (stop id)
/// - "intermediates" (array of stop ids)
#[derive(Debug)]
pub(crate) struct TransferPatternsTable(pub(crate) DataFrame);

impl TransferPatternsTable {
    pub(crate) fn new() -> PolarsResult<Self> {
        let start_col = Column::new_empty("start".into(), &DataType::UInt32);
        let target_col = Column::new_empty("target".into(), &DataType::UInt32);
        let intermediate_stops = Column::new_empty("intermediates".into(), &DataType::List(Box::new(DataType::UInt32)));

        Ok(Self(DataFrame::new(vec![start_col, target_col, intermediate_stops])?))
    }

    pub(crate) fn add(&mut self, result: RangeOutput) -> PreprocessingResult<()> {
        for journey in result.journeys {
            self.add_journey(journey)?;
        }
        Ok(())
    }

    pub(crate) fn add_journey(&mut self, journey: Journey) -> PreprocessingResult<()> {
        // All these IDs are still with cluster-local IDs. They will be replaced in `rename_stops`.
        let start_id = *journey.departure_stop();
        let start_val: AnyValue = start_id.into();
        let target_id = *journey.arrival_stop();
        let target_val: AnyValue = target_id.into();
        
        let intermediates: Series = Series::from_iter(journey.legs()
            .skip(1) // Skip first leg. For the last one, we will just take its departure, so skipping its arrival
            .map(|l| l.start().0));
        let intermediates = AnyValue::List(intermediates);

        let row = Row::new(vec![start_val, target_val, intermediates]);
        let df = DataFrame::from_rows(&[row])?;
        
        self.0.vstack_mut_unchecked(&df);

        Ok(())
    }
    
    pub(crate) fn reduce(&mut self) -> PreprocessingResult<()> {        
        // Remove duplicate transfer patterns
        self.0 = self.0.unique::<PolarsError, PolarsError>(None, UniqueKeepStrategy::Any, None)?;
        
        Ok(())
    }
}