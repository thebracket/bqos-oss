use crate::influx::*;
use chrono::{DateTime, FixedOffset, Local};
use influxdb2_structmap::FromMap;
use rocket::serde::{json::Json, Serialize};

#[derive(influxdb2_structmap_derive::FromMap, Debug)]
pub struct InternetBandwidth {
    pub site: String,
    pub field: String,
    pub value: f64,
    pub time: DateTime<FixedOffset>,
}

impl Default for InternetBandwidth {
    fn default() -> Self {
        Self {
            site: String::new(),
            field: String::new(),
            value: 0.0,
            time: chrono::MIN_DATETIME.with_timezone(&chrono::FixedOffset::east(7 * 3600)),
        }
    }
}

impl PartialEq for InternetBandwidth {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

impl PartialOrd for InternetBandwidth {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.time.partial_cmp(&other.time)
    }
}

#[derive(Serialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct InternetBandwidthRest {
    pub up: f64,
    pub down: f64,
    pub time: String,
}

impl From<&[InternetBandwidth]> for InternetBandwidthRest {
    fn from(n: &[InternetBandwidth]) -> Self {
        Self {
            time: chrono::DateTime::<Local>::from(n[0].time).to_rfc3339(),
            up: n[1].value,
            down: n[0].value,
        }
    }
}

pub async fn site_bandwidth_query(
    id: &str,
    range: &str,
    aggregate: &str,
) -> anyhow::Result<Vec<InternetBandwidthRest>> {
    let rest = InfluxQuery::new()
        .with_range_from_url(range)
        .with_aggregate_from_url(aggregate, AggregateFunction::Max)
        .with_measurement("queues")
        .with_filter(QueryFilter::Either {
            field: "_field".to_string(),
            value_1: "down_mbps".to_string(),
            value_2: "up_mbps".to_string(),
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

#[get("/query/site_bandwidth/<id>/<range>/<aggregate>")]
pub async fn site_bandwidth(
    id: String,
    range: String,
    aggregate: String,
) -> Json<Vec<InternetBandwidthRest>> {
    Json(site_bandwidth_query(&id, &range, &aggregate).await.unwrap())
}

#[get("/query/peak_bandwidth/<id>")]
pub async fn peak_bandwidth(id: String) -> Json<InternetBandwidthRest> {
    use influxdb2::{models::Query, Client};
    let qs = format!(
        "from(bucket: \"bracketqos\")
    |> range(start: -7d)
    |> filter(fn: (r) => r[\"_measurement\"] == \"queues\")
    |> filter(fn: (r) => r[\"_field\"] == \"down_mbps\" or r[\"_field\"] == \"up_mbps\")
    |> filter(fn: (r) => r[\"site\"] == \"{id}\")
    |> aggregateWindow(every: 14d, fn: max, createEmpty: false)
    |> yield(name: \"max\")
    "
    );
    let query = Query::new(qs.to_string());
    let cfg = crate::configuration();
    let client = Client::new(cfg.influx_url, cfg.influx_org, cfg.influx_token);
    let mut result: Vec<InternetBandwidth> = client
        .query::<InternetBandwidth>(Some(query))
        .await
        .unwrap();
    //println!("{:?}", result);
    result.sort_by(|a, b| b.time.cmp(&a.time));
    let rest = result
        .chunks(2)
        .map(|n| InternetBandwidthRest {
            time: chrono::DateTime::<Local>::from(n[0].time).to_rfc3339(),
            up: n[1].value,
            down: n[0].value,
        })
        .next()
        .unwrap();
    Json(rest)
}
