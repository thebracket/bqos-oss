use crate::influx::*;
use chrono::{DateTime, FixedOffset};
use influxdb2_structmap::FromMap;
use rocket::serde::{json::Json, Serialize};

#[derive(influxdb2_structmap_derive::FromMap)]
pub struct CpuLoad {
    value: f64,
    time: DateTime<FixedOffset>,
}

impl Default for CpuLoad {
    fn default() -> Self {
        Self {
            value: 0.0,
            time: chrono::MIN_DATETIME.with_timezone(&chrono::FixedOffset::east(7 * 3600)),
        }
    }
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CpuLoadRest {
    usage: f64,
}

#[get("/query/cpu_load")]
pub async fn last_cpu_average() -> Json<Vec<CpuLoadRest>> {
    let result = InfluxQuery::new()
        .with_range_string("start: -5m")
        .with_aggregate("5m", AggregateFunction::Last)
        .with_measurement("cpu")
        .with_filter(QueryFilter::MatchOne {
            field: "_field".to_string(),
            value: "usage".to_string(),
        })
        .with_group("cpu")
        .with_last()
        .run::<CpuLoad>()
        .await
        .unwrap()
        .0
        .iter()
        .map(|c| CpuLoadRest { usage: c.value })
        .collect::<Vec<CpuLoadRest>>();

    Json(result)
}
