/*use std::fmt::Debug;
use async_trait::async_trait;
use crate::data_preprocessing::step3_validate_data::rule_severity::{FatalSeverity, Severity};

pub fn gtfs_rules() -> Vec<Box<dyn Rule<impl Severity>>> {
    vec![
        Box::new(UniqueRouteIds),
        Box::new(UniqueStopIds),
    ]
}

#[async_trait]
pub trait Rule<S: Severity>: Debug {
    //async fn get_violations(&self, ctx: &SessionContext) -> Result<DataFrame>;
}

#[derive(Debug)]
struct UniqueRouteIds;

#[async_trait]
impl Rule<FatalSeverity> for UniqueRouteIds {
    /*async fn get_violations(&self, ctx: &SessionContext) -> Result<DataFrame> {
        Ok(ctx.table("routes").await?.get_and_count_duplicated("route_id".into(), "COUNT(routes.route_id)")?)
    }*/
}

#[derive(Debug)]
struct UniqueStopIds;
#[async_trait]
impl Rule<FatalSeverity> for UniqueStopIds {
/*async fn get_violations(&self, ctx: &SessionContext) -> Result<DataFrame> {
    Ok(ctx.table("stops").await?.get_and_count_duplicated("stop_id".into(), "COUNT(stops.stop_id)")?)
}*/
}

#[derive(Debug)]
struct UniqueAgencyIds;
#[async_trait]
impl Rule<FatalSeverity> for UniqueAgencyIds {
/*async fn get_violations(&self, ctx: &SessionContext) -> Result<DataFrame> {
    Ok(ctx.table("agency").await?.get_and_count_duplicated("agency_id".into(), "COUNT(agency.agency_id)")?)
}*/
}*/