use crate::influx::*;
use crate::queries::{InternetBandwidth, InternetBandwidthRest};
use anyhow::Result;
use rocket::serde::json::Json;

pub async fn site_drops_query(
    id: &str,
    range: &str,
    aggregate: &str,
) -> Result<Vec<InternetBandwidthRest>> {
    let rest = InfluxQuery::new()
        .with_range_from_url(range)
        .with_aggregate_from_url(aggregate, AggregateFunction::Max)
        .with_measurement("queues")
        .with_filter(QueryFilter::Either {
            field: "_field".to_string(),
            value_1: "down_drops".to_string(),
            value_2: "up_drops".to_string(),
        })
        .with_filter(QueryFilter::MatchOne {
            field: "site".to_string(),
            value: id.to_string(),
        })
        .run::<InternetBandwidth>()
        .await?
        .sort()
        .0
        .chunks(2)
        .map(|n| InternetBandwidthRest::from(n))
        .collect();

    Ok(rest)
}

#[get("/query/site_drops/<id>/<range>/<aggregate>")]
pub async fn site_drops(
    id: String,
    range: String,
    aggregate: String,
) -> Json<Vec<InternetBandwidthRest>> {
    Json(site_drops_query(&id, &range, &aggregate).await.unwrap())
}
