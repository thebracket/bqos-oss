use crate::{
    bus::{get_queue_tree, get_tree_children, get_tree_node_by_id},
    queries::LatencySummary,
};
use rocket::serde::{json::Json, Serialize};
use rocket::{futures::future::join_all, tokio::join};
use shared_rest::QueueTreeEntry;

#[derive(Serialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct SiteLatency {
    pub id: String,
    pub name: String,
    pub median: f64,
    pub worst: f64,
}

#[get("/reports/site_latency")]
pub async fn site_tcp_latency() -> Json<Vec<SiteLatency>> {
    let mut futures_median = Vec::new();
    let mut futures_peak = Vec::new();
    let sites: Vec<QueueTreeEntry> = get_queue_tree()
        .iter()
        .filter(|t| t.level_type == "tower")
        .cloned()
        .collect();

    for s in sites.iter() {
        futures_median.push(median_latency(s.id.clone()));
        futures_peak.push(peak_latency(s.id.clone()));
    }

    let (median_usage, peak_usage) = join!(join_all(futures_median), join_all(futures_peak));
    let latency: Vec<(f64, f64)> = median_usage
        .iter()
        .zip(peak_usage.iter())
        .map(|(m, p)| (*m, *p))
        .collect();
    //let median_usage = join_all(futures_median).await;
    //let peak_usage = join_all(futures_peak).await;

    let mut result: Vec<SiteLatency> = sites
        .iter()
        .zip(latency.iter())
        .map(|(s, (median, worst))| SiteLatency {
            id: s.id.clone(),
            name: s.name.clone(),
            median: *median,
            worst: *worst,
        })
        .collect();

    result.sort_by(|a, b| b.median.partial_cmp(&a.median).unwrap());

    Json(result)
}

#[get("/reports/ap_latency")]
pub async fn ap_tcp_latency() -> Json<Vec<SiteLatency>> {
    let mut futures_median = Vec::new();
    let mut futures_peak = Vec::new();
    let sites: Vec<QueueTreeEntry> = get_queue_tree()
        .iter()
        .filter(|t| t.level_type == "ap")
        .cloned()
        .collect();

    for s in sites.iter() {
        futures_median.push(median_latency(s.id.clone()));
        futures_peak.push(peak_latency(s.id.clone()));
    }

    let (median_usage, peak_usage) = join!(join_all(futures_median), join_all(futures_peak));
    let latency: Vec<(f64, f64)> = median_usage
        .iter()
        .zip(peak_usage.iter())
        .map(|(m, p)| (*m, *p))
        .collect();
    //let median_usage = join_all(futures_median).await;
    //let peak_usage = join_all(futures_peak).await;

    let mut result: Vec<SiteLatency> = sites
        .iter()
        .zip(latency.iter())
        .map(|(s, (median, worst))| SiteLatency {
            id: s.id.clone(),
            name: s.name.clone(),
            median: *median,
            worst: *worst,
        })
        .collect();

    result.sort_by(|a, b| b.median.partial_cmp(&a.median).unwrap());

    Json(result)
}

#[get("/reports/client_latency")]
pub async fn client_tcp_latency() -> Json<Vec<SiteLatency>> {
    let mut futures_median = Vec::new();
    let mut futures_peak = Vec::new();
    let sites: Vec<QueueTreeEntry> = get_queue_tree()
        .iter()
        .filter(|t| t.level_type == "client")
        .cloned()
        .collect();

    for s in sites.iter() {
        futures_median.push(median_latency(s.id.clone()));
        futures_peak.push(peak_latency(s.id.clone()));
    }

    let (median_usage, peak_usage) = join!(join_all(futures_median), join_all(futures_peak));
    let latency: Vec<(f64, f64)> = median_usage
        .iter()
        .zip(peak_usage.iter())
        .map(|(m, p)| (*m, *p))
        .collect();
    //let median_usage = join_all(futures_median).await;
    //let peak_usage = join_all(futures_peak).await;

    let mut result: Vec<SiteLatency> = sites
        .iter()
        .zip(latency.iter())
        .map(|(s, (median, worst))| SiteLatency {
            id: s.id.clone(),
            name: s.name.clone(),
            median: *median,
            worst: *worst,
        })
        .collect();

    result.sort_by(|a, b| b.median.partial_cmp(&a.median).unwrap());

    Json(result)
}

pub async fn peak_latency(id: String) -> f64 {
    use influxdb2::{models::Query, Client};
    let filter = build_site_filter(&id);
    let qs = format!(
        "from(bucket: \"bracketqos\")
    |> range(start: -1d)
    |> filter(fn: (r) => r[\"_measurement\"] == \"latency\")
    |> filter(fn: (r) => r[\"_field\"] == \"latency\")
    |> filter(fn: (r) => {filter})
    |> aggregateWindow(every: 1d, fn: max, createEmpty: false)
    |> yield(name: \"max\")
    "
    );
    let query = Query::new(qs.to_string());
    let cfg = crate::configuration();
    let client = Client::new(cfg.influx_url, cfg.influx_org, cfg.influx_token);
    let result: Vec<LatencySummary> = client.query::<LatencySummary>(Some(query)).await.unwrap();

    if !result.is_empty() {
        result[0].value
    } else {
        0.0
    }
}

pub async fn median_latency(id: String) -> f64 {
    use influxdb2::{models::Query, Client};
    let filter = build_site_filter(&id);
    let qs = format!(
        "from(bucket: \"bracketqos\")
    |> range(start: -1d)
    |> filter(fn: (r) => r[\"_measurement\"] == \"latency\")
    |> filter(fn: (r) => r[\"_field\"] == \"latency\")
    |> filter(fn: (r) => {filter})
    |> aggregateWindow(every: 2d, fn: median, createEmpty: false)
    |> yield(name: \"median\")
    "
    );
    let query = Query::new(qs.to_string());
    let cfg = crate::configuration();
    let client = Client::new(cfg.influx_url, cfg.influx_org, cfg.influx_token);
    let result: Vec<LatencySummary> = client
        .query::<LatencySummary>(Some(query))
        .await
        .unwrap_or(Vec::new());

    if !result.is_empty() {
        result[0].value
    } else {
        0.0
    }
}

fn build_site_filter(id: &str) -> String {
    let node = get_tree_node_by_id(id);
    if node.level_type == "client" {
        format!("r[\"site\"] == \"{id}\"")
    } else {
        // FIXME
        let mut query = format!("r[\"site\"] == \"{id}\"");
        recurse_kids(id, &mut query);
        //println!("{query}");
        query
    }
}

fn recurse_kids(id: &str, query: &mut String) {
    for n in get_tree_children(id).iter() {
        if n.level_type == "client" {
            *query = format!("{query} or r[\"site\"] == \"{}\"", n.id);
        }
        recurse_kids(&n.id, query);
    }
}
