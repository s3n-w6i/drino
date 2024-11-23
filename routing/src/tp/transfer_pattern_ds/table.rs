use crate::journey::Journey;
use crate::stp::preprocessing::clustering::filter_for_cluster::StopIdMapping;
use polars::datatypes::DataType;
use polars::error::PolarsResult;
use polars::frame::DataFrame;
use polars::prelude::Column;

#[derive(Debug)]
pub struct TransferPatternsTable(pub(crate) DataFrame);

impl TransferPatternsTable {
    pub(crate) fn new() -> PolarsResult<Self> {
        let target_col = Column::new_empty("target".into(), &DataType::UInt32);
        let start_col = Column::new_empty("start".into(), &DataType::UInt32);
        let intermediate_stops = Column::new_empty("intermediate_stops".into(), &DataType::UInt32);

        Ok(Self(DataFrame::new(vec![target_col, intermediate_stops, start_col])?))
    }
    
    pub(crate) fn add_journey(journey: Journey, stop_id_mapping: &StopIdMapping) {
        let journey_start_idx = journey.departure_stop().0 as usize;
        let journey_end = *journey.arrival_stop();
        
        //let new_rows = df!();
        todo!()
    }
}