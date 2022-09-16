use crate::{
    pretty::{display_action, display_warning},
    tree_builder::is_ip_relevant_no_igore,
};
use config::QosConfig;
use lazy_static::*;
use parking_lot::RwLock;
use shared_rest::{LatencyReport, MinMaxAvg};
use std::{
    collections::{HashMap, HashSet},
    net::Ipv4Addr,
    str::FromStr,
};
use tokio::{process::Command, spawn};

lazy_static! {
    static ref IP_TO_CLIENT_SITE: RwLock<HashMap<Ipv4Addr, String>> = RwLock::new(HashMap::new());
}

lazy_static! {
    static ref UNMAPPED_IP: RwLock<HashSet<Ipv4Addr>> = RwLock::new(HashSet::new());
}

struct LatencyResult {
    latency: f32,
}

struct LatencyMap {
    latencies: HashMap<String, Vec<LatencyResult>>,
}

impl LatencyMap {
    fn new() -> Self {
        Self {
            latencies: HashMap::new(),
        }
    }

    fn store_site(&mut self, site: &str, latency: f32) {
        if let Some(l) = self.latencies.get_mut(site) {
            l.push(LatencyResult { latency });
        } else {
            self.latencies
                .insert(site.to_string(), vec![LatencyResult { latency }]);
        }
    }

    fn store(&mut self, ip: Ipv4Addr, latency: f32) {
        let lock = IP_TO_CLIENT_SITE.read();
        if let Some(cs) = lock.get(&ip) {
            self.store_site(&cs, latency);
        }
    }

    fn store_latency(&mut self, ip_1: &str, ip_2: &str, rtt1: f32, _rtt2: f32) {
        let latency = f32::min(rtt1, 200.0);
        let ip_1 = Ipv4Addr::from_str(ip_1).unwrap_or(Ipv4Addr::UNSPECIFIED);
        let ip_2 = Ipv4Addr::from_str(ip_2).unwrap_or(Ipv4Addr::UNSPECIFIED);

        let lock = IP_TO_CLIENT_SITE.read();
        let mut stored = false;

        let is_ip_1_local = lock.contains_key(&ip_1);
        let is_ip_2_local = lock.contains_key(&ip_2);
        let are_both_local = is_ip_1_local && is_ip_2_local;

        if !are_both_local {
            if is_ip_1_local {
                self.store(ip_1, latency);
                stored = true;
            }
            if is_ip_2_local {
                self.store(ip_2, latency);
                stored = true;
            }

            if !stored {
                //println!("Unmapped IP pair: {}/{}", ip_1, ip_2);
                if is_ip_relevant_no_igore(ip_1) {
                    //println!("{ip_1}");
                    UNMAPPED_IP.write().insert(ip_1);
                }
                if is_ip_relevant_no_igore(ip_2) {
                    //println!("{ip_2}");
                    UNMAPPED_IP.write().insert(ip_2);
                }
            }
        }
    }

    fn to_latency_report(&mut self) -> LatencyReport {
        let mut result = LatencyReport::new();
        for (k, v) in self.latencies.iter().filter(|(_, v)| v.len() > 2) {
            let latency = MinMaxAvg {
                average: v.iter().map(|l| l.latency).sum::<f32>() / v.len() as f32,
            };

            result.report(&k, latency);
        }
        let mut lock = UNMAPPED_IP.write();
        result.add_unmapped(&lock);
        lock.clear();
        self.latencies.clear();
        result
    }
}

pub fn map_ip_to_site(ip: &str, site: &str) {
    //println!("Mapping {ip} to {site}");
    if let Ok(ip) = Ipv4Addr::from_str(ip) {
        let mut lock = IP_TO_CLIENT_SITE.write();
        lock.insert(ip, site.to_string());
    }
}

pub async fn gather_latency(config: &QosConfig) {
    let mut latency_map = LatencyMap::new();
    loop {
        let pping = Command::new("/usr/local/bin/pping")
            .arg("-i")
            .arg(&config.to_internet)
            .arg("-s")
            .arg("60")
            .arg("-m")
            .output()
            .await;
        //tokio::time::sleep(Duration::from_secs(60)).await;
        if let Ok(pping) = pping {
            display_action("Latency Check Result", 2);
            if let Ok(raw) = std::str::from_utf8(&pping.stdout) {
                for line in raw.split('\n') {
                    let fields = line.split(' ').collect::<Vec<&str>>();
                    if fields.len() == 7 {
                        //let timestamp = fields[0];
                        let rtt1 = fields[1].parse::<f32>().unwrap_or(0.0) * 1_000.0;
                        let rtt2 = fields[2].parse::<f32>().unwrap_or(0.0) * 1_000.0;
                        let ip_raw = fields[6];
                        let ip_split = ip_raw.split(&[':', '+']).collect::<Vec<&str>>();
                        if ip_split.len() == 4 {
                            let ip_1 = ip_split[0];
                            let ip_2 = ip_split[2];

                            latency_map.store_latency(ip_1, ip_2, rtt1, rtt2);
                        }
                    }
                }
                let report = latency_map.to_latency_report();
                //println!("{:#?}", report);
                spawn(send_report(
                    report,
                    format!("{}/bus/latency", &config.controller_url),
                ));
            } else {
                display_warning("Unable to read latency stdout", 2);
            }
        } else {
            display_warning("Latency Check Failed", 2);
        }
    }
}

async fn send_report(report: LatencyReport, url: String) {
    let client = reqwest::Client::new();
    let res = client.post(&url).json(&report).send().await;
    if res.is_err() {
        println!("{:?}", res);
    }
}
