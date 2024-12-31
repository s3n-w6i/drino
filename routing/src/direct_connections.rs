use geoarrow::table::Table;
use itertools::Itertools;
use polars::frame::DataFrame;
use polars::prelude::*;
use polars::series::IntoSeries;

use crate::algorithm::{PreprocessingError, PreprocessingInput};
use common::types::StopId;
use common::util::df;
use common::util::geoarrow_lines::build_geoarrow_lines;

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
/// | line_id | trip_id      | stop_id | arrival | departure | stop_sequence |
/// | ------- | ------------ | ------- | ------- | --------- | ------------- |
/// | 0       | 0            | 42      | null    | 8:15      |             0 |
/// | 0       | 0            | 7       | 8:22    | 8:23      |             1 |
/// | 0       | 0            | 68      | 8:38    | 8:39      |             2 |
/// | 0       | 0            | ...     | ...     | ...       |           ... |
/// | 0       | 1            | ...     | ...     | ...       |             0 |
/// | 0       | ...          | ...     | ...     | ...       |           ... |
/// | 1       | ...          | ...     | ...     | ...       |           ... |
/// | ...     | ...          | ...     | ...     | ...       |           ... |
pub type ExpandedLinesFrame = DataFrame;

/// | line_id | stop_id  | stop_sequence   |
/// | ------- | -------- | --------------- |
/// | 0       | 5        | 0               |
/// | 0       | 2        | 1               |
/// | 1       | 3        | 0               |
/// | 1       | ...      | ...             |
/// | ...     | ...      | ...             |
pub type LineProgressionFrame = DataFrame;

/// | stop_id | incidences                                                     |
/// | ------- | -------------------------------------------------------------- |
/// | 0       | [(line_id=42, stop_sequence=2), (line_id=15, stop_sequence=7)] |
/// | 1       | ...                                                            |
pub type StopIncidenceFrame = DataFrame;


#[derive(Clone, Debug)]
pub struct DirectConnections {
    pub expanded_lines: ExpandedLinesFrame,
    pub line_progressions: LineProgressionFrame,
    pub stop_incidence: StopIncidenceFrame,
}


impl TryFrom<PreprocessingInput> for DirectConnections {
    type Error = PreprocessingError;

    fn try_from(input: PreprocessingInput) -> Result<Self, Self::Error> {
        let (expanded_lines, line_progressions) = {
            // TODO: For now, this completely ignores traffic days. Therefore, computed transfer patterns might include some patterns that are never possible and might not include some optimal ones (when mixture of days is better than whats possible on an actual day)!
            let mut lines = input.stop_times
                .clone()
                // Sort the stop sequence, so that list of stop_ids are identical once aggregated
                .sort(["stop_sequence"], Default::default())
                // Turn the stop_ids into a list per each trip
                .group_by([col("trip_id")])
                .agg([col("stop_id").alias("stop_ids"), col("arrival_time"), col("departure_time"), col("stop_sequence")])
                // Group by the sequence of stop_ids, to identify lines (aka unique sequences of stops)
                .group_by([col("stop_ids"), col("stop_sequence")])
                .agg([col("trip_id").alias("trip_ids"), col("arrival_time"), col("departure_time")])
                .collect()?;

            // Assign line ids
            let num_lines = lines.get_columns().first().unwrap().len() as u32;
            let mut line_ids = Column::from(Series::from_iter(0..num_lines));
            line_ids.rename("line_id".into());
            lines.with_column(line_ids)?;

            let line_progressions = lines.clone().lazy()
                .select([col("line_id"), col("stop_ids"), col("stop_sequence")])
                .explode([col("stop_ids"), col("stop_sequence")])
                .rename(["stop_ids"], ["stop_id"], true); // TODO

            let exploded_lines = lines.lazy()
                // Disaggregate trips of a line
                .explode([col("trip_ids"), col("arrival_time"), col("departure_time")])
                // Rename plural "trip_ids" back to singular "trip_id"
                .select([col("*").exclude(["trip_ids"]), col("trip_ids").alias("trip_id")])
                // Disaggregate stops of a trip
                .explode([col("stop_ids"), col("arrival_time"), col("departure_time"), col("stop_sequence")])
                // Rename plural "stop_ids" back to singular "stop_id"
                .select([col("*").exclude(["stop_ids"]), col("stop_ids").alias("stop_id")]);

            Ok::<(ExpandedLinesFrame, LineProgressionFrame), PreprocessingError>(
                (exploded_lines.collect()?, line_progressions.collect()?)
            )
        }?;

        let stop_incidence = {
            let incidences: Series = expanded_lines.clone()
                .select(["line_id", "stop_sequence"])?
                .into_struct("incidences".into())
                .into_series();

            expanded_lines
                .select(["stop_id"])?
                .with_column(incidences)?
                .clone().lazy()
                .unique(None, UniqueKeepStrategy::Any)
                .group_by([col("stop_id")])
                .agg([col("incidences")])
        }.collect()?;

        Ok(Self { expanded_lines, line_progressions, stop_incidence })
    }
}

/// Custom implementation of equality. This is needed, since the column order could be different for
/// dataframes. They are still equivalent, but a strict eq would fail.
impl PartialEq for DirectConnections {
    fn eq(&self, other: &Self) -> bool {
        df::equivalent(&self.expanded_lines, &other.expanded_lines, true, true).unwrap()
            && df::equivalent(&self.line_progressions, &other.line_progressions, true, true).unwrap()
            && df::equivalent(&self.stop_incidence, &other.stop_incidence, true, true).unwrap()
    }
}


impl DirectConnections {
    pub(crate) fn query_direct(&self, from: StopId, to: StopId) -> Result<LazyFrame, PreprocessingError> {
        // Utility function to filter for incidences whose stop_id matches
        fn filter_and_unpack_incidences(StopId(id): StopId, stop_incidence: &StopIncidenceFrame) -> Result<LazyFrame, PreprocessingError> {
            let filter_mask = stop_incidence.column("stop_id")?.as_materialized_series().equal(id)?;
            Ok(stop_incidence
                .filter(&filter_mask)?
                .select(["incidences"])?
                .clone().lazy() // todo: remove this clone
                .explode([col("incidences")])
                .unnest(["incidences"])
            )
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

    pub(crate) fn query_direct_earliest_after(
        &self, from: StopId, to: StopId, departure: Duration,
    ) -> Result<LazyFrame, PreprocessingError> {
        let common_lines = self.query_direct(from, to)?;
        let common_lines_after_departure = common_lines
            .left_join(
                self.expanded_lines.select(["line_id", "departure_time"])?.lazy(),
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

    pub fn to_geoarrow_lines(
        &self,
        stops_df: LazyFrame,
    ) -> Result<Table, common::util::geoarrow_lines::Error> {
        let stop_chains = self.line_progressions.clone().lazy()
            .sort(["stop_sequence"], SortMultipleOptions::default())
            .group_by([col("line_id")])
            .agg([col("stop_id")])
            .select([col("stop_id")])
            .collect()?;

        let [stop_chains] = stop_chains.get_columns()
        else { unreachable!("we only selected a single column") };

        let stop_chains: Vec<Vec<StopId>> = stop_chains.list()?.into_iter()
            .map(|l| {
                let l = l.unwrap();
                let l = l.u32().unwrap();
                l.into_iter().map(|id| StopId::from(id.unwrap())).collect_vec()
            })
            .collect_vec();

        let table = build_geoarrow_lines(
            stop_chains,
            stops_df,
        )?;

        Ok(table)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::case_1::generate_preprocessing_input;
    use chrono::{NaiveDateTime, TimeDelta};
    use polars::datatypes::AnyValue::List;

    #[test]
    fn test_case_1() {
        let input = generate_preprocessing_input().unwrap();

        let expected = DirectConnections {
            line_progressions: df![
                "line_id" => [0u32, 0],
                "stop_id" => [0u32, 1],
                "stop_sequence" => [0u32, 1],
            ].unwrap(),
            expanded_lines: df![
                "line_id" => [0u32, 0],
                "trip_id" => [0u32, 0],
                "stop_id" => [0u32, 1],
                "arrival" => [NaiveDateTime::UNIX_EPOCH + TimeDelta::seconds(100), NaiveDateTime::UNIX_EPOCH + TimeDelta::seconds(500)],
                "departure" => [NaiveDateTime::UNIX_EPOCH + TimeDelta::seconds(100), NaiveDateTime::UNIX_EPOCH + TimeDelta::seconds(500)],
                "stop_sequence" => [0u32, 1],
            ].unwrap(),
            stop_incidence: df![
                "stop_id" => [0u32, 1],
                "incidences" => [
                    List(
                        df![
                            "line_id" => [0u32],
                            "stop_sequence" => [0u32],
                        ].unwrap().into_struct("_".into()).into_series()
                    ),
                    List(
                        df![
                            "line_id" => [0u32],
                            "stop_sequence" => [1u32],
                        ].unwrap().into_struct("_".into()).into_series()
                    ),
                ]
            ].unwrap(),
        };
        let actual = DirectConnections::try_from(input).unwrap();

        assert_eq!(expected, actual);
    }
}
