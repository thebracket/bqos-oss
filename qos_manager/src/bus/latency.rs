use chrono::{DateTime, Local};
use lazy_static::*;
use parking_lot::RwLock;
use rocket::{futures::stream, serde::json::Json};
use std::{collections::HashMap, time::Duration};

lazy_static! {
    pub static ref UNMAPPED_IP: RwLock<HashMap<String, chrono::DateTime<Local>>> =
        RwLock::new(HashMap::new());
}

#[post("/bus/latency", data = "<latency>")]
pub async fn latency_report(latency: Json<shared_rest::LatencyReport>) {
    //println!("{:#?}", latency);

    if let Some(mut lock) = UNMAPPED_IP.try_write_for(Duration::from_secs(2)) {
        let local: DateTime<Local> = Local::now();
        for unmapped in latency.unmapped_ip.iter() {
            if let Some(t) = lock.get_mut(unmapped) {
                *t = local;
            } else {
                lock.insert(unmapped.to_string(), local);
            }
        }
        let yesterday = local - chrono::Duration::hours(24);
        lock.retain(|_, date| *date > yesterday);
    }

    use influxdb2::models::DataPoint;
    use influxdb2::Client;

    let mut tmp = Vec::new();
    for line in latency.items.iter() {
        for site_id in crate::bus::get_parent_ids(&line.site) {
            tmp.push(
                DataPoint::builder("latency")
                    .tag("site", &site_id)
                    .field("latency", line.latency.average as f64)
                    .build()
                    .unwrap(),
            );
        }
    }

    let cfg = crate::configuration();
    let client = Client::new(cfg.influx_url, cfg.influx_org, cfg.influx_token);
    let result = client.write("bracketqos", stream::iter(tmp)).await;
    if result.is_err() {
        println!("{:?}", result);
    }
}
