use super::LatencySummaryRest;
use crate::influx::*;
use anyhow::Result;
use chrono::{DateTime, FixedOffset, Local};
use influxdb2_structmap::FromMap;
use rocket::serde::json::Json;

#[derive(influxdb2_structmap_derive::FromMap, Debug)]
pub struct FrequencySummary {
    pub value: f64,
    pub time: DateTime<FixedOffset>,
}

impl Default for FrequencySummary {
    fn default() -> Self {
        Self {
            value: 0.0,
            time: chrono::MIN_DATETIME.with_timezone(&chrono::FixedOffset::east(7 * 3600)),
        }
    }
}

pub async fn ap_frequency_query(
    id: &str,
    range: &str,
    aggregate: &str,
) -> Result<Vec<LatencySummaryRest>> {
    let rest = InfluxQuery::new()
        .with_range_from_url(range)
        .with_aggregate_from_url(aggregate, AggregateFunction::Max)
        .with_measurement("frequency")
        .with_filter(QueryFilter::MatchOne {
            field: "access_point".to_string(),
            value: id.to_string(),
        })
        .run::<FrequencySummary>()
        .await?
        .0
        .iter()
        .map(|n| LatencySummaryRest {
            date: chrono::DateTime::<Local>::from(n.time).to_rfc3339(),
            avg: n.value,
            min: n.value,
            max: n.value,
        })
        .collect();
    Ok(rest)
}

pub async fn ap_noise_query(
    id: &str,
    range: &str,
    aggregate: &str,
) -> Result<Vec<LatencySummaryRest>> {
    let rest = InfluxQuery::new()
        .with_range_from_url(range)
        .with_aggregate_from_url(aggregate, AggregateFunction::Max)
        .with_measurement("noise_floor")
        .with_filter(QueryFilter::MatchOne {
            field: "access_point".to_string(),
            value: id.to_string(),
        })
        .run::<FrequencySummary>()
        .await?
        .0
        .iter()
        .map(|n| LatencySummaryRest {
            date: chrono::DateTime::<Local>::from(n.time).to_rfc3339(),
            avg: n.value,
            min: n.value,
            max: n.value,
        })
        .collect();

    Ok(rest)
}

pub async fn signal_query(
    id: &str,
    range: &str,
    aggregate: &str,
) -> Result<Vec<LatencySummaryRest>> {
    let rest = InfluxQuery::new()
        .with_range_from_url(range)
        .with_aggregate_from_url(aggregate, AggregateFunction::Max)
        .with_measurement("signal")
        .with_filter(QueryFilter::MatchOne {
            field: "access_point".to_string(),
            value: id.to_string(),
        })
        .run::<FrequencySummary>()
        .await?
        .0
        .iter()
        .map(|n| LatencySummaryRest {
            date: chrono::DateTime::<Local>::from(n.time).to_rfc3339(),
            avg: n.value,
            min: n.value,
            max: n.value,
        })
        .collect();

    Ok(rest)
}

#[get("/query/ap_frequency/<id>/<range>/<aggregate>")]
pub async fn ap_frequency(
    id: String,
    range: String,
    aggregate: String,
) -> Json<Vec<LatencySummaryRest>> {
    Json(ap_frequency_query(&id, &range, &aggregate).await.unwrap())
}

#[get("/query/ap_noise/<id>/<range>/<aggregate>")]
pub async fn ap_noise(
    id: String,
    range: String,
    aggregate: String,
) -> Json<Vec<LatencySummaryRest>> {
    Json(ap_noise_query(&id, &range, &aggregate).await.unwrap())
}

#[get("/query/signal/<id>/<range>/<aggregate>")]
pub async fn signal(id: String, range: String, aggregate: String) -> Json<Vec<LatencySummaryRest>> {
    Json(signal_query(&id, &range, &aggregate).await.unwrap())
}
