use crate::AppData;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use routing::algorithms::errors::{QueryError, QueryResult};
use routing::algorithms::queries::cardinality::{All, Multiple, Single, TargetCardinality};
use routing::algorithms::queries::range::{Range, RangeOutput};
use routing::algorithms::queries::{QueryType, Queryable};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// TODO: This feels like it should not need to be defined manually. Macro?
#[derive(Deserialize)]
#[serde(untagged)]
pub enum AnyQuery<'a> {
    //EaSingle(Query<EarliestArrival, Single>),
    //EaMulti(Query<EarliestArrival, Multiple<'a>>),
    //EaAll(Query<EarliestArrival, All>),
    //LdSingle(Query<LatestDeparture, Single>),
    //LdMulti(Query<LatestDeparture, Single>),
    //LdAll(Query<LatestDeparture, All>),
    //RangeSingle(Query<Range, Single>),
    //RangeMulti(Query<Range, Multiple<'a>>),
    RangeAll(Query<'a, Range>),
}

#[derive(Deserialize)]
pub struct Query<'a, QT: QueryType> {
    #[serde(flatten)]
    pub input: QT::Input,
    #[serde(flatten)]
    pub target_cardinality: AnyTargetCardinality<'a>,
}

#[derive(Deserialize)]
#[serde(tag = "target_type", rename_all = "snake_case")]
pub enum AnyTargetCardinality<'a> {
    Single(Single),
    Multiple(Multiple<'a>),
    All(All),
}

impl<'a> TryFrom<AnyTargetCardinality<'a>> for Single {
    type Error = QueryError;

    fn try_from(value: AnyTargetCardinality<'a>) -> Result<Self, Self::Error> {
        match value {
            AnyTargetCardinality::Single(s) => Ok(s),
            _ => Err(QueryError::InvalidTargetCardinality),
        }
    }
}

impl<'a> TryFrom<AnyTargetCardinality<'a>> for Multiple<'a> {
    type Error = QueryError;

    fn try_from(value: AnyTargetCardinality<'a>) -> Result<Self, Self::Error> {
        match value {
            AnyTargetCardinality::Multiple(m) => Ok(m),
            _ => Err(QueryError::InvalidTargetCardinality),
        }
    }
}

impl<'a> TryFrom<AnyTargetCardinality<'a>> for All {
    type Error = QueryError;

    fn try_from(value: AnyTargetCardinality<'a>) -> Result<Self, Self::Error> {
        match value {
            AnyTargetCardinality::All(a) => Ok(a),
            _ => Err(QueryError::InvalidTargetCardinality),
        }
    }
}

pub(crate) async fn endpoint(
    State(app_data): State<Arc<AppData>>,
    axum::extract::Query(query): axum::extract::Query<Query<'_, Range>>,
) -> Result<Json<RangeOutput>, (StatusCode, String)> {
    let algorithm = &app_data.algorithm;

    /*let result = match query.0 {
        //AnyQuery::EaSingle(q) => to_responder(run::<EarliestArrival, Single, _>(algorithm, q)),
        //AnyQuery::EaMulti(q) => run::<EarliestArrival, Multiple>(algorithm, q),
        //AnyQuery::EaAll(q) => run::<EarliestArrival, All>(algorithm, q),
        //AnyQuery::LdSingle(q) => run::<LatestDeparture, Single>(algorithm, q),
        //AnyQuery::LdMulti(q) => run(algorithm, q),
        //AnyQuery::LdAll(q) => run(algorithm, q),
        //AnyQuery::RangeSingle(q) => to_responder(run::<Range, Single, _>(algorithm, q)),
        //AnyQuery::RangeMulti(q) => run::<Range, Multiple>(algorithm, q),
        AnyQuery::RangeAll(q) => run::<Range, All, _>(algorithm, q),
    };*/

    let result = run2::<Range, All, _>(algorithm, query);

    result.map(|r| Json(r)).map_err(|err| convert_error(err))
}

// Utility function to run a generic query on an algorithm
fn run2<'a, QT, TC, R>(algorithm: &impl Queryable<QT, TC>, query: Query<'a, QT>) -> QueryResult<R>
where
    QT: QueryType,
    TC: TargetCardinality<QT, Output = R> + TryFrom<AnyTargetCardinality<'a>, Error = QueryError>,
    R: Serialize,
{
    algorithm.query(query.input, query.target_cardinality.try_into()?)
}

fn convert_error(err: QueryError) -> (StatusCode, String) {
    match err {
        QueryError::Polars(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        QueryError::NoRouteFound => (StatusCode::NOT_FOUND, QueryError::NoRouteFound.to_string()),
        QueryError::TransferError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        QueryError::InvalidTargetCardinality => (
            StatusCode::BAD_REQUEST,
            QueryError::InvalidTargetCardinality.to_string(),
        ),
    }
}
