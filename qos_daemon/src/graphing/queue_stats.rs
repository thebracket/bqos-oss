use crate::{pretty::display_action, shaper::TC_CMD};
use anyhow::Result;
use chrono::{DateTime, Utc};
use config::QosConfig;
use lazy_static::*;
use parking_lot::RwLock;
use shared_rest::{BandwidthLine, BandwidthReport};
use std::{
    collections::HashMap,
    time::{Duration, Instant, SystemTime},
};
use tokio::{join, process::Command, spawn};

lazy_static! {
    static ref QUEUE_TO_CLIENT_SITE: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
}

lazy_static! {
    static ref HTB_QUEUE_TO_CLIENT_SITE: RwLock<HashMap<String, String>> =
        RwLock::new(HashMap::new());
}

pub fn map_queue_to_site(queue: (u32, u32), site: &str) {
    let mut lock = QUEUE_TO_CLIENT_SITE.write();
    lock.insert(format!("{}:{}", queue.0, queue.1), site.to_string());
}

pub fn map_htb_queue_to_site(queue: (u32, u32), site: &str) {
    let mut lock = HTB_QUEUE_TO_CLIENT_SITE.write();
    lock.insert(format!("{}:{}", queue.0, queue.1), site.to_string());
}

lazy_static! {
    static ref DOWNLOAD: RwLock<HashMap<String, InterfaceStats>> = RwLock::new(HashMap::new());
}

lazy_static! {
    static ref UPLOAD: RwLock<HashMap<String, InterfaceStats>> = RwLock::new(HashMap::new());
}

struct InterfaceStats {
    last_query: SystemTime,
    drops: u64,
    bytes: u64,
    prior_drops: u64,
    prior_bytes: u64,
}

fn read_bytes_and_drops(
    q: &serde_json::Value,
    is_download: bool,
    new_time: &SystemTime,
    map: &str,
) {
    if let Some(drops) = q.get("drops") {
        if let Some(bytes_sent) = q.get("bytes") {
            let drops = drops.as_u64().unwrap_or(0);
            let bytes = bytes_sent.as_u64().unwrap_or(0);

            let mut lock = if is_download {
                DOWNLOAD.write()
            } else {
                UPLOAD.write()
            };
            if let Some(cs) = lock.get_mut(map) {
                cs.last_query = new_time.clone();
                cs.prior_drops = cs.drops;
                cs.prior_bytes = cs.bytes;
                cs.drops = drops;
                cs.bytes = bytes;
            } else {
                lock.insert(
                    map.to_string(),
                    InterfaceStats {
                        last_query: new_time.clone(),
                        drops,
                        bytes,
                        prior_bytes: 0,
                        prior_drops: 0,
                    },
                );
            }

            //if bytes_sent > 0 {
            //    println!("Mapped {} to {map} => drop:{}, packet:{}. bytes:{}", parent, drops, packets, bytes_sent);
            //}
        }
    }
}

async fn queue_stats(interface: &str, is_download: bool, new_time: SystemTime) -> Result<()> {
    let out = Command::new(TC_CMD)
        .arg("-j")
        .arg("-s")
        .arg("qdisc")
        .arg("show")
        .arg("dev")
        .arg(interface)
        .output()
        .await?;

    let raw_queue_stats = std::str::from_utf8(&out.stdout)?;
    let json: Vec<serde_json::Value> = serde_json::from_str(&format!("{raw_queue_stats}"))?;

    for q in json.iter() {
        if let Some(parent) = q.get("parent") {
            if let Some(parent) = parent.as_str() {
                if let Some(map) = QUEUE_TO_CLIENT_SITE.read().get(parent) {
                    read_bytes_and_drops(q, is_download, &new_time, &map);
                }
            }
        } else {
            if let Some(handle) = q.get("handle") {
                if handle == "7fff:" {
                    read_bytes_and_drops(q, is_download, &new_time, "root");
                }
            } else {
                println!("No handle or mapped parent detected");
                println!("{:?}", q);
            }
        }
    }

    //println!("{:#?}", json);
    Ok(())
}

// Not using this anymore, because querying by class gave wildly innacurrate results and
// hit the CPU far too hard. Preserving out of interest.
/*#[deprecated]
fn class_result(is_download: bool, new_time: &SystemTime, queue: &str, bytes: u64, drops: u64) {
    if let Some(map) = HTB_QUEUE_TO_CLIENT_SITE.read().get(queue) {
        let mut lock = if is_download {
            DOWNLOAD.write()
        } else {
            UPLOAD.write()
        };
        if let Some(cs) = lock.get_mut(map) {
            cs.last_query = new_time.clone();
            cs.prior_drops = cs.drops;
            cs.prior_bytes = cs.bytes;
            cs.drops = drops;
            cs.bytes = bytes;
        } else {
            lock.insert(
                map.to_string(),
                InterfaceStats {
                    last_query: new_time.clone(),
                    drops,
                    bytes,
                    prior_bytes: 0,
                    prior_drops: 0,
                },
            );
        }
    }
}*/

/*async fn class_stats(interface: &str, is_download: bool, new_time: SystemTime) -> Result<()> {
    //println!("Gathering for {interface}");
    let out = Command::new(TC_CMD)
        .arg("-s")
        .arg("class")
        .arg("show")
        .arg("dev")
        .arg(interface)
        .output()
        .await?;

    let raw_queue_stats = std::str::from_utf8(&out.stdout)?;
    let lines: Vec<&str> = raw_queue_stats.split('\n').collect();
    let mut current_queue = String::new();
    for line in lines.iter() {
        // Did we start a new one?
        if line.starts_with("class htb") {
            if let Some(idx) = line.find("parent") {
                current_queue = line[10..idx].trim().to_string();
            }
        }
        // If we have a queue, do we have some data?
        if !current_queue.is_empty() && line.starts_with(" Sent") {
            //println!("{line}");
            let mut bytes = 0;
            let mut dropped = 0;
            if let Some(idx) = line.find("bytes") {
                let bytes_str = &line[6..idx];
                if let Ok(b) = bytes_str.trim().parse::<u64>() {
                    bytes = b;
                }
            }
            if let Some(dropped_idx) = line.find("dropped") {
                if let Some(comma_idx) = line.find(',') {
                    let dropped_str = &line[dropped_idx..comma_idx];
                    if let Ok(d) = dropped_str.trim().parse::<u64>() {
                        dropped = d;
                    }
                }
            }
            class_result(is_download, &new_time, &current_queue, bytes, dropped);
        }
    }

    Ok(())
}*/

pub async fn gather_interface_stats(config: &QosConfig) {
    // Gather queue data every minute
    let mut last_check: Option<std::time::Instant> = None;
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
        display_action("Polling Queue Counts", 2);
        let time = SystemTime::now();
        let down_stats = queue_stats(&config.to_isp, true, time.clone());
        let up_stats = queue_stats(&config.to_internet, false, time.clone());
        //let down_class_stats = class_stats(&config.to_isp, true, time.clone());
        //let up_class_stats = class_stats(&config.to_internet, false, time.clone());
        //let _ = join!(down_stats, up_stats, down_class_stats, up_class_stats);
        let _ = join!(down_stats, up_stats);

        let time_formatter: DateTime<Utc> = time.into();
        let mut report = BandwidthReport::new(time_formatter.format("%+").to_string());
        for (site_id, stats) in DOWNLOAD.read().iter() {
            let bytes = if stats.bytes > stats.prior_bytes {
                stats.bytes - stats.prior_bytes
            } else {
                0
            } as f64;
            let bytes_per_second = bytes / 60.0;
            let mbits_per_second = (bytes_per_second / 1_000_000.0) * 8.0;

            let drops = if stats.drops > stats.prior_drops {
                stats.drops - stats.prior_drops
            } else {
                0
            };

            report.download.push(BandwidthLine {
                site_id: site_id.clone(),
                mbits_per_second,
                drops,
            });
        }
        for (site_id, stats) in UPLOAD.read().iter() {
            let bytes = if stats.bytes > stats.prior_bytes {
                stats.bytes - stats.prior_bytes
            } else {
                0
            } as f64;
            let seconds_elapsed = if let Some(last_check) = last_check {
                last_check.elapsed().as_secs_f64()
            } else {
                60.0
            };
            let bytes_per_second = bytes / seconds_elapsed;
            let mbits_per_second = (bytes_per_second / 1_000_000.0) * 8.0;

            let drops = if stats.drops > stats.prior_drops {
                stats.drops - stats.prior_drops
            } else {
                0
            };

            report.upload.push(BandwidthLine {
                site_id: site_id.clone(),
                mbits_per_second,
                drops,
            });
        }
        if last_check.is_some() {
            // TODO: Walk trees upwards now we have all of the child numbers.
            add_parent_bandwidth(&mut report);
            spawn(send_report(
                report,
                format!("{}/bus/bandwidth", &config.controller_url),
            ));
        }
        last_check = Some(Instant::now());
    }
}

fn add_parent_bandwidth(report: &mut BandwidthReport) {
    let tree = crate::tree_builder::QUEUE_SUMMARY.read();
    let mut add_dl = Vec::new();
    let mut add_up = Vec::new();

    for dl_host in report.download.iter() {
        walk_queue_tree(&tree, dl_host, &mut add_dl);
    }
    for ul_host in report.upload.iter() {
        walk_queue_tree(&tree, ul_host, &mut add_up);
    }

    report.download.extend_from_slice(&add_dl);
    report.upload.extend_from_slice(&add_up);
}

fn walk_queue_tree(
    tree: &[shared_rest::QueueTreeEntry],
    line: &BandwidthLine,
    target: &mut Vec<BandwidthLine>,
) {
    if let Some(tree_entry) = tree.iter().find(|qs| qs.id == line.site_id) {
        let mut current = tree_entry.parent;
        while current.is_some() {
            let current_queue = &tree[current.unwrap()];
            if let Some(bw) = target.iter_mut().find(|b| b.site_id == current_queue.id) {
                bw.mbits_per_second += line.mbits_per_second;
                bw.drops += line.drops;
            } else {
                target.push(BandwidthLine {
                    site_id: current_queue.id.to_string(),
                    mbits_per_second: line.mbits_per_second,
                    drops: line.drops,
                });
            }
            current = current_queue.parent;
        }
    }
}

async fn send_report(report: BandwidthReport, url: String) {
    let client = reqwest::Client::new();
    let _ = client.post(&url).json(&report).send().await;
}
