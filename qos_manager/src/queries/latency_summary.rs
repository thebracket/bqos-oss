use chrono::{DateTime, FixedOffset, Local};
use influxdb2_structmap::FromMap;
use rocket::{
    serde::{json::Json, Serialize},
    tokio::join,
};

#[derive(influxdb2_structmap_derive::FromMap, Debug)]
pub struct LatencySummary {
    pub value: f64,
    pub time: DateTime<FixedOffset>,
}

impl Default for LatencySummary {
    fn default() -> Self {
        Self {
            value: 0.0,
            time: chrono::MIN_DATETIME.with_timezone(&chrono::FixedOffset::east(7 * 3600)),
        }
    }
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct LatencySummaryRest {
    pub avg: f64,
    pub min: f64,
    pub max: f64,
    pub date: String,
}

async fn calc_latency(id: &str, range: &str, aggregate: &str, mode: &str) -> Vec<LatencySummary> {
    use influxdb2::{models::Query, Client};
    let range = urlencoding::decode(&range).unwrap();
    let aggregate = urlencoding::decode(&aggregate).unwrap();
    //println!("{range}, {aggregate}");

    // Get Average
    let filter = format!("r[\"site\"] == \"{id}\"");
    let qs = format!(
        "from(bucket: \"bracketqos\")
    |> range({range})
    |> filter(fn: (r) => r[\"_measurement\"] == \"latency\")
    |> filter(fn: (r) => r[\"_field\"] == \"latency\")
    |> filter(fn: (r) => {filter})
    |> group(columns: [\"_measurement\"])
    |> aggregateWindow(every: {aggregate}, fn: {mode}, createEmpty: false)
    |> yield(name: \"{mode}\")
    "
    );
    //println!("{qs}");
    let query = Query::new(qs.to_string());
    let cfg = crate::configuration();
    let client = Client::new(cfg.influx_url, cfg.influx_org, cfg.influx_token);
    let mut average: Vec<LatencySummary> =
        client.query::<LatencySummary>(Some(query)).await.unwrap();

    // Sort by time
    average.sort_by(|a, b| a.time.cmp(&b.time));
    average
}

#[get("/query/latency_site/<id>/<range>/<aggregate>")]
pub async fn latency_site(
    id: String,
    range: String,
    aggregate: String,
) -> Json<Vec<LatencySummaryRest>> {
    let (average, min, max) = join!(
        calc_latency(&id, &range, &aggregate, "median"),
        calc_latency(&id, &range, &aggregate, "min"),
        calc_latency(&id, &range, &aggregate, "max"),
    );

    let max_index = usize::min(average.len(), usize::min(min.len(), max.len()));
    let mut rest = Vec::new();
    for i in 0..max_index {
        let local_time = chrono::DateTime::<Local>::from(average[i].time);
        rest.push(LatencySummaryRest {
            date: local_time.to_rfc3339(),
            avg: average[i].value,
            min: min[i].value,
            max: max[i].value,
        })
    }

    Json(rest)
}
