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

#[get("/query/ram_use")]
pub async fn last_ram_use() -> Json<Vec<CpuLoadRest>> {
    let result = InfluxQuery::new()
        .with_range_string("start: -15m")
        .with_aggregate("5m", AggregateFunction::Mean)
        .with_measurement("memory")
        .with_group("cpu")
        .run::<CpuLoad>()
        .await
        .unwrap()
        .0
        .iter()
        .map(|c| CpuLoadRest { usage: c.value })
        .collect::<Vec<CpuLoadRest>>();

    Json(result)
}

#[get("/query/swap_use")]
pub async fn last_swap_use() -> Json<Vec<CpuLoadRest>> {
    use influxdb2::{models::Query, Client};
    let qs = "from(bucket: \"bracketqos\")
    |> range(start: -15m)
    |> filter(fn: (r) => r[\"_measurement\"] == \"swap\")
    |> aggregateWindow(every: 5m, fn: mean, createEmpty: false)
    |> last()
    ";
    let query = Query::new(qs.to_string());
    let cfg = crate::configuration();
    let client = Client::new(cfg.influx_url, cfg.influx_org, cfg.influx_token);
    let result: Vec<CpuLoad> = client.query::<CpuLoad>(Some(query)).await.unwrap();
    let result = result
        .iter()
        .map(|c| CpuLoadRest { usage: c.value })
        .collect::<Vec<CpuLoadRest>>();
    Json(result)
}
