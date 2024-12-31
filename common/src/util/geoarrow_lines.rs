use crate::types::{f64_from_any_value, StopId};
use arrow_array::RecordBatch;
use arrow_schema::{ArrowError, Schema, SchemaRef};
use geo::{coord, LineString};
use geoarrow::array::LineStringBuilder;
use geoarrow::datatypes::Dimension;
use geoarrow::error::GeoArrowError;
use geoarrow::table::Table;
use geoarrow::ArrayBase;
use polars::error::PolarsError;
use polars::prelude::{col, LazyFrame};
use std::fmt::Display;

pub fn build_geoarrow_lines(
    stop_chains: Vec<Vec<StopId>>,
    stops_df: LazyFrame
) -> Result<Table, Error> {
    let stop_locations = stops_df
        .select([col("stop_id"), col("lat"), col("lon")])
        .collect()?;
    let stop_locations = stop_locations.get_columns();
    let stop_id_series = stop_locations[0].as_materialized_series();
    let stop_ids = stop_id_series.u32()?;
    
    let mut builder: LineStringBuilder = LineStringBuilder::new(Dimension::XY);

    for stops in stop_chains {
        let locations = stops.into_iter().map(|stop| {
            let idx = &stop_ids.iter().position(|s| s.unwrap() == stop.0)
                .unwrap_or_else(|| panic!("stop {} not found in provided stops_df", stop.0));
            
            let lat = stop_locations[1].get(*idx).unwrap();
            let lon = stop_locations[2].get(*idx).unwrap();
            
            (
                f64_from_any_value(lat).unwrap(),
                f64_from_any_value(lon).unwrap(),
            )
        });
        let coordinates = locations
            .into_iter()
            .map(|(lat, lon)| coord! { x: lon, y: lat })
            .collect();

        builder.push_line_string(Some(&LineString::new(coordinates)))?
    }
    let array = builder.finish();
    let field = array.extension_field();
    let schema: SchemaRef = Schema::new(vec![field]).into();
    let columns = vec![array.into_array_ref()];
    let batch = RecordBatch::try_new(schema.clone(), columns)?;
    let table = Table::try_new(vec![batch], schema)?;
    
    Ok(table)
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    Polars(#[from] PolarsError),
    Geoarrow(#[from] GeoArrowError),
    Arrow(#[from] ArrowError)
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Polars(e) => e.fmt(f),
            Error::Geoarrow(e) => e.fmt(f),
            Error::Arrow(e) => e.fmt(f)
        }
    }
}