use polars::frame::DataFrame;
use polars::prelude::{ChunkCompare, col, IntoLazy, LazyFrame, lit, Series};
use polars::series::IntoSeries;

use crate::algorithm::{PreprocessingError, PreprocessingInput};
use common::types::StopId;

#[derive(Clone)]
pub struct DirectConnections {
    pub lines: LinesFrame,
    pub stop_incidence: StopIncidenceFrame,
}

impl TryFrom<PreprocessingInput> for DirectConnections {
    type Error = PreprocessingError;

    fn try_from(input: PreprocessingInput) -> Result<Self, Self::Error> {
        let lines = create_line_table(&input)?.collect()?;
        let stop_incidence = create_stop_incidence_table(&lines)?.collect()?;
        Ok(Self { lines, stop_incidence })
    }
}

pub type LinesFrame = DataFrame;


/// In the transfer patterns paper, lines are represented like this:
///
/// | Line 17 | Stop 042 | Stop 007  | Stop 068  | ... |
/// | ------- | -------- | --------- | --------- | --- |
/// | trip 1  | 8:15     | 8:22 8:23 | 8:38 8:39 | ... |
/// | trip 2  | 9:14     | 9:21 8:23 | 9:38 9:40 | ... |
/// | ...     | ...      | ...       | ...       | ... |
///
/// In this implementation, all lines are stored in a single table, and stop times are represented
/// vertically instead of horizontally, like this:
///
/// | line_id | trip_id      | stop_id | arrival | departure |
/// | ------- | ------------ | ------- | ------- | --------- |
/// | 0       | 0            | 42      | null    | 8:15      |
/// | 0       | 0            | 7       | 8:22    | 8:23      |
/// | 0       | 0            | 68      | 8:38    | 8:39      |
/// | 0       | 0            | ...     | ...     | ...       |
/// | 0       | 1            | ...     | ...     | ...       |
/// | 0       | ...          | ...     | ...     | ...       |
/// | 1       | ...          | ...     | ...     | ...       |
/// | ...     | ...          | ...     | ...     | ...       |
// TODO: For now, this completely ignores traffic days. Therefore, computed transfer patterns might include some patterns that are never possible and might not include some optimal ones (when mixture of days is better than whats possible on an actual day)!
fn create_line_table(PreprocessingInput { stop_times, .. }: &PreprocessingInput) -> Result<LazyFrame, PreprocessingError> {
    let lines = stop_times
        .clone()
        // Sort the stop sequence, so that list of stop_ids are identical once aggregated
        .sort(["stop_sequence"], Default::default())
        // Turn the stop_ids into a list per each trip
        .group_by([col("trip_id")])
        .agg([col("stop_id").alias("stop_ids"), col("arrival_time"), col("departure_time"), col("stop_sequence")])
        .group_by([col("stop_ids")])
        .agg([col("trip_id").alias("trip_ids"), col("arrival_time"), col("departure_time"), col("stop_sequence")])
        // Assign line ids
        .with_row_index("line_id", None);

    let exploded_lines = lines
        // Disaggregate trips of a line
        .explode([col("trip_ids"), col("arrival_time"), col("departure_time"), col("stop_sequence")])
        // Rename plural "trip_ids" back to singular "trip_id"
        .select([col("*").exclude(["trip_ids"]), col("trip_ids").alias("trip_id")])
        // Disaggregate stops of a trip
        .explode([col("stop_ids"), col("arrival_time"), col("departure_time"), col("stop_sequence")])
        // Rename plural "stop_ids" back to singular "stop_id"
        .select([col("*").exclude(["stop_ids"]), col("stop_ids").alias("stop_id")]);

    Ok(exploded_lines)
}

pub type StopIncidenceFrame = DataFrame;

fn create_stop_incidence_table(lines: &DataFrame) -> Result<LazyFrame, PreprocessingError> {
    let incidences: Series = lines.clone()
        .select(["line_id", "stop_sequence"])?
        .into_struct("incidences".into())
        .into_series();
    let stop_incidence_frame = lines
        .select(["stop_id"])?
        .with_column(incidences)?
        .clone().lazy()
        .group_by([col("stop_id")])
        .agg([col("incidences")]);

    Ok(stop_incidence_frame)
}

impl DirectConnections {
    fn query_direct(&self, from: StopId, to: StopId) -> Result<LazyFrame, PreprocessingError> {
        // Utility function to filter for incidences whose stop_id matches
        fn filter_and_unpack_incidences(StopId(id): StopId, stop_incidence: &StopIncidenceFrame) -> Result<LazyFrame, PreprocessingError> {
            let filter_mask = stop_incidence.column("stop_id")?.equal(id)?;
            Ok(stop_incidence
                .filter(&filter_mask)?
                .select(["incidences"])?
                .clone().lazy() // todo: remove this clone
                .explode([col("incidences")])
                .unnest(["incidences"]))
        }

        // Only select those that start at from_stop
        let from_incidences = filter_and_unpack_incidences(from, &self.stop_incidence)?;
        // Only select those that end at to_stop
        let to_incidences = filter_and_unpack_incidences(to, &self.stop_incidence)?;

        let common_lines = from_incidences
            .select([col("line_id"), col("stop_sequence").alias("from_sequence_num")])
            .inner_join(
                to_incidences.select([
                    col("line_id"), col("stop_sequence").alias("to_sequence_num")
                ]),
                "line_id", "line_id",
            );

        let common_lines_where_sequence_correct = common_lines
            .filter(col("from_sequence_num").lt(col("to_sequence_num")));
        Ok(common_lines_where_sequence_correct)
    }

    fn query_direct_earliest_after(&self, from: StopId, to: StopId, departure: polars::prelude::Duration) -> Result<LazyFrame, PreprocessingError> {
        let common_lines = self.query_direct(from, to)?;
        let common_lines_after_departure = common_lines
            .left_join(
                self.lines.select(["line_id", "departure_time"])?.lazy(),
                col("line_id"), col("line_id"),
            )
            .filter(
                col("departure_time").gt_eq(lit(departure.nanoseconds() / 1_000_000))
            );
        let earliest = common_lines_after_departure
            .sort(["departure_time"], Default::default())
            .first();
        Ok(earliest)
    }
}