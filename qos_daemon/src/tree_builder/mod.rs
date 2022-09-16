use std::collections::HashSet;

use anyhow::Result;
use shared_rest::{DuplicateIp, QueueTreeEntry};
use tokio::spawn;
use uisp_support::{DataLink, Device, Site};
mod ip_matchers;
mod queue_tree;
use config::{QosConfig, ShapingStrategy};
pub use queue_tree::*;
mod strategy;
use crate::pretty::display_warning;
pub use ip_matchers::{is_ip_relevant_no_igore, load_ip_matching};
use lazy_static::*;
use parking_lot::RwLock;

lazy_static! {
    pub static ref QUEUE_SUMMARY: RwLock<Vec<QueueTreeEntry>> = RwLock::new(Vec::new());
}

/// Sends sites, devices and data-links to the appropriate strategy builder (defined in the config)
pub async fn build_logical_tree(
    config: &QosConfig,
    sites: &[Site],
    devices: &[Device],
    data_links: &[DataLink],
) -> Result<QueueTree> {
    // Create a tree containing only top-level per-CPU items.
    let mut tree = QueueTree::new(config).await?;

    // Call the appropriate strategy builder to define a queue tree (defined in the config)
    match config.strategy {
        ShapingStrategy::JustClients => {
            strategy::single_layer_strategy(config, &mut tree, sites, devices).await?;
        }
        ShapingStrategy::SiteOnly => {
            strategy::site_only_strategy(config, &mut tree, sites, devices).await?;
        }
        ShapingStrategy::Full => {
            strategy::full_tree_hierarchy(config, &mut tree, sites, devices, data_links).await?;
        }
    };

    // Traverse the tree, propagating maximums. For example, if Site 1 has a maximum speed
    // of 10 Mbps, no entry beneath it may have a limit of more than 10 Mbps.
    set_tree_maximums(&mut tree, config);

    // Search the generated tree for duplicate IP addresses. Older versions of UISP made it
    // far too easy to have some of these (newer ones do some really weird stuff when it happens,
    // but it probably won't reach this program!)
    let dupes = check_for_duplicate_ips(&tree);
    
    // If there are any duplicates, submit the list to the manager
    if !dupes.is_empty() {
        send_dupes(
            dupes,
            format!("{}/bus/duplicate_ip", &config.controller_url),
        )
        .await;
    }

    // Create the queue tree summary in the REST format required
    // by the manager. Store it (for future sends) and send it
    // to the manager.
    let tree_summary = tree.to_monitor_tree(config);
    *QUEUE_SUMMARY.write() = tree_summary.clone();
    spawn(send_tree_report(
        tree_summary,
        format!("{}/bus/tree", &config.controller_url),
    ));

    Ok(tree)
}

// Send the duplicate IP list via REST to the manager.
async fn send_dupes(dupes: Vec<String>, url: String) {
    let report = DuplicateIp { dupes };
    let client = reqwest::Client::new();
    let res = client.post(&url).json(&report).send().await;
    if res.is_err() {
        println!("{:?}", res);
    }
}

/// Walk the tree, collecting IP addresses. If duplicates are encountered,
/// list them.
fn check_for_duplicate_ips(tree: &QueueTree) -> Vec<String> {
    let mut ips = HashSet::new();
    let mut dupes = Vec::new();
    for q in tree.queues.iter() {
        tree_ip_walk(q, &mut ips, &mut dupes);
    }
    if !dupes.is_empty() {
        display_warning(&format!("{} duplicate IPs reported", dupes.len()), 3);
    }
    dupes
}

fn tree_ip_walk(queue: &Queue, ips: &mut HashSet<String>, dupes: &mut Vec<String>) {
    match &queue.queue_type {
        QueueType::ClientSite { ip_addresses, .. } => {
            for ip in ip_addresses.iter() {
                if ips.contains(&*ip) {
                    dupes.push(ip.to_string());
                } else {
                    ips.insert(ip.to_string());
                }
            }
        }
        _ => {}
    }
    for q in queue.children.iter() {
        tree_ip_walk(q, ips, dupes);
    }
}

fn set_tree_maximums(tree: &mut QueueTree, config: &QosConfig) {
    for q in tree.queues.iter_mut() {
        let speed_limit = match q.queue_type {
            QueueType::AccessPointSite {
                down_mbps, up_mbps, ..
            } => (down_mbps, up_mbps),
            QueueType::ClientSite {
                down_mbps, up_mbps, ..
            } => (down_mbps, up_mbps),
            QueueType::TowerSite {
                down_mbps, up_mbps, ..
            } => (down_mbps, up_mbps),
            QueueType::CpuQueue { .. } => {
                (config.internet_download_mbps, config.internet_upload_mbps)
            }
        };
        walk_tree_maximums(q, speed_limit, config);
    }
}

fn walk_tree_maximums(queue: &mut Queue, speed_limit: (u32, u32), config: &QosConfig) {
    let my_speed_limit = match queue.queue_type {
        QueueType::AccessPointSite {
            down_mbps, up_mbps, ..
        } => (down_mbps, up_mbps),
        QueueType::ClientSite {
            down_mbps, up_mbps, ..
        } => (down_mbps, up_mbps),
        QueueType::TowerSite {
            down_mbps, up_mbps, ..
        } => (down_mbps, up_mbps),
        QueueType::CpuQueue { .. } => (config.internet_download_mbps, config.internet_upload_mbps),
    };
    let speed_limit = (
        u32::min(my_speed_limit.0, speed_limit.0),
        u32::min(my_speed_limit.1, speed_limit.1),
    );
    match &mut queue.queue_type {
        QueueType::AccessPointSite {
            down_mbps, up_mbps, ..
        } => {
            *down_mbps = speed_limit.0;
            *up_mbps = speed_limit.1;
        }
        QueueType::ClientSite {
            down_mbps, up_mbps, ..
        } => {
            *down_mbps = speed_limit.0;
            *up_mbps = speed_limit.1;
        }
        QueueType::TowerSite {
            down_mbps, up_mbps, ..
        } => {
            *down_mbps = speed_limit.0;
            *up_mbps = speed_limit.1;
        }
        QueueType::CpuQueue { .. } => {}
    };
    for q in queue.children.iter_mut() {
        walk_tree_maximums(q, speed_limit, config);
    }
}

/// Send the queue tree (in manager friendly format) to the manager
/// program.
async fn send_tree_report(report: Vec<QueueTreeEntry>, url: String) {
    let client = reqwest::Client::new();
    let res = client.post(&url).json(&report).send().await;
    if res.is_err() {
        println!("{:?}", res);
    }
}
