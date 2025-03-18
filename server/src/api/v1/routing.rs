use crate::{AppData, ALGORITHM};
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use routing::algorithms::errors::{QueryError, QueryResult};
use routing::algorithms::queries;
use routing::algorithms::queries::cardinality::{All, Single, TargetCardinality};
use routing::algorithms::queries::earliest_arrival::EarliestArrival;
use routing::algorithms::queries::{Query, QueryType, Queryable};
use serde::{Deserialize, Serialize};
use common::types::config::Config;
use routing::algorithms::queries::range::Range;

// TODO: This feels like it should not need to be defined manually. Macro?
#[derive(Deserialize)]
#[serde(untagged)]
enum AnyQuery {
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

#[get("/api/v1/routing")]
pub(crate) async fn endpoint(
    app_data: web::Data<AppData>,
    //query: web::Query<AnyQuery>,
) -> actix_web::Result<impl Responder> {
    //let algorithm = &app_data.algorithm;

    /*let result = match query.0 {
        //AnyQuery::EaSingle(q) => to_responder(run::<EarliestArrival, Single, _>(algorithm, q)),
        //AnyQuery::EaMulti(q) => run::<EarliestArrival, Multiple>(algorithm, q),
        //AnyQuery::EaAll(q) => run::<EarliestArrival, All>(algorithm, q),
        //AnyQuery::LdSingle(q) => run::<LatestDeparture, Single>(algorithm, q),
        //AnyQuery::LdMulti(q) => run(algorithm, q),
        //AnyQuery::LdAll(q) => run(algorithm, q),
        //AnyQuery::RangeSingle(q) => to_responder(run::<Range, Single, _>(algorithm, q)),
        //AnyQuery::RangeMulti(q) => run::<Range, Multiple>(algorithm, q),
        AnyQuery::RangeAll(q) => to_responder(run::<Range, All, _>(algorithm, q)),
        _ => HttpResponse::NotImplemented().body("This is not a supported query type"),
    };*/

    println!("AppData address: {:?}", std::ptr::addr_of!(app_data));

    //Ok(result)
    Ok(HttpResponse::Ok().body("Hi"))
}

fn run<QT, TC, R>(algorithm: &impl Queryable<QT, TC>, query: Query<QT, TC>) -> QueryResult<R>
where
    QT: QueryType,
    TC: TargetCardinality<QT, Output = R>,
    R: Serialize
{
    queries::run(algorithm, query)
}

fn to_responder<A: Serialize>(result: Result<A, QueryError>) -> HttpResponse {
    match result {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(QueryError::NoRouteFound) => HttpResponse::NotFound().body("No route found"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}
