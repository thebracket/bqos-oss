use crate::{
    bus::get_queue_tree,
    queries::{InternetBandwidth, InternetBandwidthRest},
};
use chrono::Local;
use rocket::futures::future::join_all;
use rocket::serde::{json::Json, Serialize};
use shared_rest::QueueTreeEntry;

#[derive(Serialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct SiteCongestion {
    pub id: String,
    pub name: String,
    pub down: f64,
    pub up: f64,
    pub down_total: u32,
    pub up_total: u32,
}

#[get("/reports/site_congestion")]
pub async fn site_congestion() -> Json<Vec<SiteCongestion>> {
    let mut futures = Vec::new();
    let sites: Vec<QueueTreeEntry> = get_queue_tree()
        .iter()
        .filter(|t| t.level_type == "tower")
        .cloned()
        .collect();

    for s in sites.iter() {
        futures.push(peak_bandwidth(s.id.clone()));
    }

    let peak_usage = join_all(futures).await;

    let mut result: Vec<SiteCongestion> = sites
        .iter()
        .zip(peak_usage.iter())
        .map(|(s, (down, up))| {
            let down_pct = *down as f64 / s.down_mbps as f64;
            let up_pct = *up as f64 / s.up_mbps as f64;
            SiteCongestion {
                id: s.id.clone(),
                name: s.name.clone(),
                down: down_pct * 100.0,
                up: up_pct * 100.0,
                down_total: *down,
                up_total: *up,
            }
        })
        .collect();

    result.sort_by(|a, b| b.down.partial_cmp(&a.down).unwrap());

    Json(result)
}

#[get("/reports/ap_congestion")]
pub async fn ap_congestion() -> Json<Vec<SiteCongestion>> {
    let mut futures = Vec::new();
    let sites: Vec<QueueTreeEntry> = get_queue_tree()
        .iter()
        .filter(|t| t.level_type == "ap")
        .cloned()
        .collect();

    for s in sites.iter() {
        futures.push(peak_bandwidth(s.id.clone()));
    }

    let peak_usage = join_all(futures).await;

    let mut result: Vec<SiteCongestion> = sites
        .iter()
        .zip(peak_usage.iter())
        .map(|(s, (down, up))| {
            let down_pct = *down as f64 / s.down_mbps as f64;
            let up_pct = *up as f64 / s.up_mbps as f64;
            SiteCongestion {
                id: s.id.clone(),
                name: s.name.clone(),
                down: down_pct * 100.0,
                up: up_pct * 100.0,
                down_total: *down,
                up_total: *up,
            }
        })
        .collect();

    result.sort_by(|a, b| b.down.partial_cmp(&a.down).unwrap());

    Json(result)
}

#[get("/reports/client_congestion")]
pub async fn client_congestion() -> Json<Vec<SiteCongestion>> {
    let mut futures = Vec::new();
    let sites: Vec<QueueTreeEntry> = get_queue_tree()
        .iter()
        .filter(|t| t.level_type == "client")
        .cloned()
        .collect();

    for s in sites.iter() {
        futures.push(peak_bandwidth(s.id.clone()));
    }

    let peak_usage = join_all(futures).await;

    let mut result: Vec<SiteCongestion> = sites
        .iter()
        .zip(peak_usage.iter())
        .map(|(s, (down, up))| {
            let down_pct = *down as f64 / s.down_mbps as f64;
            let up_pct = *up as f64 / s.up_mbps as f64;
            SiteCongestion {
                id: s.id.clone(),
                name: s.name.clone(),
                down: down_pct * 100.0,
                up: up_pct * 100.0,
                down_total: *down,
                up_total: *up,
            }
        })
        .collect();

    result.sort_by(|a, b| b.down.partial_cmp(&a.down).unwrap());

    Json(result)
}

async fn peak_bandwidth(id: String) -> (u32, u32) {
    use influxdb2::{models::Query, Client};
    let qs = format!(
        "from(bucket: \"bracketqos\")
    |> range(start: -1d)
    |> filter(fn: (r) => r[\"_measurement\"] == \"queues\")
    |> filter(fn: (r) => r[\"_field\"] == \"down_mbps\" or r[\"_field\"] == \"up_mbps\")
    |> filter(fn: (r) => r[\"site\"] == \"{id}\")
    |> aggregateWindow(every: 2d, fn: max, createEmpty: false)
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
    if let Some(rest) = result
        .chunks(2)
        .map(|n| InternetBandwidthRest {
            time: chrono::DateTime::<Local>::from(n[0].time).to_rfc3339(),
            up: n[1].value,
            down: n[0].value,
        })
        .next()
    {
        (rest.down as u32, rest.up as u32)
    } else {
        (0, 0)
    }
}
