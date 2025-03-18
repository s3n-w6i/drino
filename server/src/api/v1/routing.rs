use crate::AppData;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use routing::algorithms::errors::{QueryError, QueryResult};
use routing::algorithms::queries;
use routing::algorithms::queries::cardinality::{All, TargetCardinality};
use routing::algorithms::queries::range::{Range, RangeOutput};
use routing::algorithms::queries::{Query, QueryType, Queryable};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// TODO: This feels like it should not need to be defined manually. Macro?
#[derive(Deserialize)]
#[serde(untagged)]
pub enum AnyQuery {
    //EaSingle(Query<EarliestArrival, Single>),
    //EaMulti(Query<EarliestArrival, Multiple<'a>>),
    //EaAll(Query<EarliestArrival, All>),
    //LdSingle(Query<LatestDeparture, Single>),
    //LdMulti(Query<LatestDeparture, Single>),
    //LdAll(Query<LatestDeparture, All>),
    //RangeSingle(Query<Range, Single>),
    //RangeMulti(Query<Range, Multiple<'a>>),
    RangeAll(Query<Range, All>),
}

pub(crate) async fn endpoint(
    State(app_data): State<Arc<AppData>>,
    query: axum::extract::Query<AnyQuery>,
) -> Result<Json<RangeOutput>, (StatusCode, String)> {
    let algorithm = &app_data.algorithm;

    let result = match query.0 {
        //AnyQuery::EaSingle(q) => to_responder(run::<EarliestArrival, Single, _>(algorithm, q)),
        //AnyQuery::EaMulti(q) => run::<EarliestArrival, Multiple>(algorithm, q),
        //AnyQuery::EaAll(q) => run::<EarliestArrival, All>(algorithm, q),
        //AnyQuery::LdSingle(q) => run::<LatestDeparture, Single>(algorithm, q),
        //AnyQuery::LdMulti(q) => run(algorithm, q),
        //AnyQuery::LdAll(q) => run(algorithm, q),
        //AnyQuery::RangeSingle(q) => to_responder(run::<Range, Single, _>(algorithm, q)),
        //AnyQuery::RangeMulti(q) => run::<Range, Multiple>(algorithm, q),
        AnyQuery::RangeAll(q) => run::<Range, All, _>(algorithm, q),
    };

    result
        .map(|r| Json(r))
        .map_err(|err| convert_error(err))
}

fn run<QT, TC, R>(algorithm: &impl Queryable<QT, TC>, query: Query<QT, TC>) -> QueryResult<R>
where
    QT: QueryType,
    TC: TargetCardinality<QT, Output = R>,
    R: Serialize,
{
    queries::run(algorithm, query)
}

fn convert_error(err: QueryError) -> (StatusCode, String) {
    match err {
        QueryError::Polars(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        QueryError::NoRouteFound => (StatusCode::NOT_FOUND, QueryError::NoRouteFound.to_string()),
        QueryError::TransferError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}
