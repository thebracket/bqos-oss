use std::collections::{HashMap, HashSet};

use crate::{
    shaper::{get_access_point_limits, get_site_limits},
    tree_builder::{ip_matchers::ip_addresses_in_site, Queue, QueueTree},
};
use anyhow::Result;
use config::QosConfig;
use shared_rest::ApLimit;
use uisp_support::{DataLink, Device, Site};

#[derive(Debug, Clone)]
struct VSite {
    name: String,
    id: String,
    parent: Option<String>,
    children: Vec<VSite>,
    infrastructure_ips: Vec<String>,
    clients: HashMap<(String, String), Vec<VClient>>,
    speed_limit: (u32, u32),
}

#[derive(Debug, Clone)]
struct VClient {
    name: String,
    id: String,
    parent: Option<String>,
    ip_addresses: Vec<String>,
    speed_limit: (u32, u32),
    access_point: (Option<String>, Option<String>),
}

pub async fn full_tree_hierarchy(
    config: &QosConfig,
    tree: &mut QueueTree,
    sites: &[Site],
    devices: &[Device],
    data_links: &[DataLink],
) -> Result<()> {
    let site_limits = get_site_limits();
    let ap_limits = get_access_point_limits();

    // Bbuild a virtual map of all client sites
    let mut clients: Vec<VClient> = sites
        .iter()
        .filter(|s| !s.is_tower())
        .map(|client| {
            let mut parent = None;
            if let Some(id) = &client.identification {
                if let Some(sp) = &id.parent {
                    if let Some(id) = &sp.id {
                        parent = Some(id.clone());
                    }
                }
            }
            // If the site has a child site as a parent, try to promote it
            if parent.is_some() {
                let parent_copy = parent.as_ref().unwrap().clone();
                sites
                    .iter()
                    .filter(|s| s.is_client_site() && s.id == parent_copy)
                    .for_each(|s| {
                        if let Some(id) = &s.identification {
                            if let Some(sp) = &id.parent {
                                if let Some(id) = &sp.id {
                                    parent = Some(id.clone());
                                }
                            }
                        }
                    });
            }

            VClient {
                name: client.name().unwrap_or("nameless".to_string()),
                id: client.id.clone(),
                parent: parent.clone(),
                ip_addresses: ip_addresses_in_site(client, devices).unwrap_or(Vec::new()),
                speed_limit: client.qos(config.default_download_mbps, config.default_upload_mbps),
                access_point: find_access_point(data_links, devices, &client.id, &parent),
            }
        })
        .collect();
    // Strip out client sites with no IP addresses
    clients.retain(|c| !c.ip_addresses.is_empty());

    // Build a virtual map of all tower sites
    let mut included_clients = HashSet::new();
    let mut towers: Vec<VSite> = sites
        .iter()
        .filter(|s| s.is_tower())
        .map(|site| {
            let mut parent = None;
            let mut proceed = true;
            if let Some(name) = site.name() {
                if name == config.root_site_name {
                    proceed = false;
                }
            }
            if proceed {
            if let Some(id) = &site.identification {
                if let Some(sp) = &id.parent {
                    if let Some(id) = &sp.id {
                        parent = Some(id.clone());
                    }
                }
            }
            }
            // If the site has a child site as a parent, try to promote it
            if parent.is_some() {
                let parent_copy = parent.as_ref().unwrap().clone();
                sites
                    .iter()
                    .filter(|s| s.is_client_site() && s.id == parent_copy)
                    .for_each(|s| {
                        if let Some(id) = &s.identification {
                            if let Some(sp) = &id.parent {
                                if let Some(id) = &sp.id {
                                    parent = Some(id.clone());
                                }
                            }
                        }
                    });
            }

            let (down_mbps, up_mbps) =
                if let Some(limit) = site_limits.iter().find(|sl| sl.id == site.id) {
                    (limit.download, limit.upload)
                } else {
                    (config.internet_download_mbps, config.internet_upload_mbps)
                };

            VSite {
                name: site.name().unwrap_or("nameless".to_string()),
                id: site.id.clone(),
                parent,
                children: Vec::new(),
                infrastructure_ips: ip_addresses_in_site(site, devices).unwrap_or(Vec::new()),
                clients: map_site_clients(&site.id, &clients, &mut included_clients),
                speed_limit: (down_mbps, up_mbps),
            }
        })
        .collect();
    // Strip out towers with no infrastructure, children or clients
    towers.retain(|t| {
        let should_remove =
            t.infrastructure_ips.is_empty() && t.children.is_empty() && t.clients.is_empty();
        !should_remove
    });

    // TODO:
    // * Stop excluding root-less sites. Move them to the top.
    // * Any towers with client sites as parents need to be handled. They will not be in included sites yet.
    // * Client to Client site links need to be handled

    // Extract the top-level sites (with no parent)
    let mut included_sites = HashSet::new();
    let mut root_sites: Vec<VSite> = towers
        .iter()
        .filter(|t| t.parent.is_none())
        .cloned()
        .collect();
    root_sites.iter().for_each(|s| {
        included_sites.insert(s.id.clone());
    });
    for root in root_sites.iter_mut() {
        map_child_sites(root, &towers, &mut included_sites);
    }
    let not_included = towers
        .iter()
        .filter(|t| !included_sites.contains(&t.id))
        .count();
    if not_included > 0 {
        println!("Warning: {not_included} sites were skipped.");
        towers
            .iter()
            .filter(|t| !included_sites.contains(&t.id))
            .for_each(|s| {
                println!("{} / {}", s.name, s.id);
            });
    }
    let not_included = clients
        .iter()
        .filter(|t| !included_clients.contains(&t.id))
        .count();
    if not_included > 0 {
        let not_included_list = clients
            .iter()
            .filter(|t| !included_clients.contains(&t.id))
            .map(|t| t.name.clone())
            .collect::<Vec<String>>();
        println!(
            "Warning: {not_included} clients weren't mapped. These will be added to a fake site."
        );
        //println!("{:#?}", not_included_list);
        send_unmapped(
            not_included_list,
            format!("{}/bus/unmapped_clients", &config.controller_url),
        )
        .await;
        let mut access_points = HashMap::new();
        access_points.insert(
            ("No AP".to_string(), "666.2".to_string()),
            clients
                .iter()
                .filter(|t| !included_clients.contains(&t.id))
                .cloned()
                .collect(),
        );
        let fake_site = VSite {
            name: "No Site Parent".to_string(),
            id: "666".to_string(),
            parent: None,
            children: Vec::new(),
            infrastructure_ips: Vec::new(),
            clients: access_points,
            speed_limit: (config.internet_download_mbps, config.internet_upload_mbps),
        };
        root_sites.push(fake_site);
    }
    //println!("{:#?}", root_sites);

    // Now that we have a virtual site map, we need to make the actual queues. Start by iterating the top-level and creating items
    let mut top_level_queue = 0;
    let max_queue = u32::min(tree.queue_count.to_isp, tree.queue_count.to_internet) as usize;
    for root in root_sites.iter() {
        if !root.infrastructure_ips.is_empty() {
            let infrastructure = Queue::new_client_site(
                &format!("{} Infrastructure", root.name),
                root.speed_limit.0,
                root.speed_limit.1,
                &root.infrastructure_ips.clone(),
                &format!("{}.1", root.id),
            );
            tree.queues[top_level_queue].children.push(infrastructure);
            top_level_queue += 1;
            top_level_queue %= max_queue;
        }

        for ((ap_name, ap_id), clients) in root.clients.iter() {
            let (down_mbps, up_mbps) = if let Some(ap) = ap_limits.iter().find(|a| a.id == *ap_id) {
                (ap.download, ap.upload)
            } else {
                root.speed_limit
            };
            let mut ap = Queue::new_access_point_site(&ap_name, down_mbps, up_mbps, &ap_id);
            for c in clients.iter() {
                let cs = Queue::new_client_site(
                    &c.name,
                    c.speed_limit.0,
                    c.speed_limit.1,
                    &c.ip_addresses,
                    &c.id,
                );
                ap.children.push(cs);
            }
            tree.queues[top_level_queue].children.push(ap);
            top_level_queue += 1;
            top_level_queue %= max_queue;
        }

        for child in root.children.iter() {
            let mut link = Queue::new_tower_site(
                &child.name,
                child.speed_limit.0,
                child.speed_limit.1,
                &child.id,
            );
            build_site_queue(child, &mut link, &ap_limits);

            // Balance accross CPUs
            tree.queues[top_level_queue].children.push(link);
            top_level_queue += 1;
            top_level_queue %= max_queue;
        }
    }

    //println!("{:#?}", tree);
    //panic!("STOP");

    Ok(())
}

fn build_site_queue(root: &VSite, link: &mut Queue, ap_limits: &[ApLimit]) {
    if !root.infrastructure_ips.is_empty() {
        let infrastructure = Queue::new_client_site(
            &format!("{} Infrastructure", root.name),
            root.speed_limit.0,
            root.speed_limit.1,
            &root.infrastructure_ips.clone(),
            &format!("{}.1", root.id),
        );
        link.children.push(infrastructure);
    }
    for ((ap_name, ap_id), clients) in root.clients.iter() {
        let (down_mbps, up_mbps) = if let Some(ap) = ap_limits.iter().find(|a| a.id == *ap_id) {
            (ap.download, ap.upload)
        } else {
            root.speed_limit
        };
        let mut ap = Queue::new_access_point_site(&ap_name, down_mbps, up_mbps, &ap_id);
        for c in clients.iter() {
            let cs = Queue::new_client_site(
                &c.name,
                c.speed_limit.0,
                c.speed_limit.1,
                &c.ip_addresses,
                &c.id,
            );
            ap.children.push(cs);
        }
        link.children.push(ap);
    }
    for child in root.children.iter() {
        let mut clink = Queue::new_tower_site(
            &child.name,
            child.speed_limit.0,
            child.speed_limit.1,
            &child.id,
        );
        build_site_queue(child, &mut clink, ap_limits);
        link.children.push(clink);
    }
}

fn map_child_sites(site: &mut VSite, sites: &[VSite], included_sites: &mut HashSet<String>) {
    sites
        .iter()
        .filter(|s| {
            if let Some(parent) = &s.parent {
                if *parent == site.id {
                    return true;
                }
            }
            false
        })
        .for_each(|s| {
            let mut new_site = s.clone();
            included_sites.insert(new_site.id.clone());
            map_child_sites(&mut new_site, sites, included_sites);
            site.children.push(new_site);
        })
}

fn map_site_clients(
    site_id: &str,
    clients: &[VClient],
    included_clients: &mut HashSet<String>,
) -> HashMap<(String, String), Vec<VClient>> {
    let mut aps: HashMap<(String, String), Vec<VClient>> = HashMap::new();

    let result: Vec<VClient> = clients
        .iter()
        .filter(|c| {
            if let Some(parent) = &c.parent {
                if *parent == site_id {
                    return true;
                }
            }
            false
        })
        .cloned()
        .collect();
    result.iter().for_each(|c| {
        included_clients.insert(c.id.clone());
    });

    for c in result.iter() {
        let ap_name = c.access_point.0.clone().unwrap_or("No AP".to_string());
        let ap_id = c.access_point.1.clone().unwrap_or(format!("{}.2", site_id));
        if let Some(ap) = aps.get_mut(&(ap_name, ap_id)) {
            ap.push(c.clone());
        } else {
            aps.insert(
                (
                    c.access_point.0.clone().unwrap_or("No AP".to_string()),
                    c.access_point.1.clone().unwrap_or(format!("{}.2", site_id)),
                ),
                vec![c.clone()],
            );
        }
    }

    aps
}

fn find_access_point(
    data_links: &[DataLink],
    devices: &[Device],
    client_id: &str,
    parent_id: &Option<String>,
) -> (Option<String>, Option<String>) {
    let mut ap_name = None;
    let mut ap_id = None;
    if let Some(parent_id) = &parent_id {
        data_links
            .iter()
            .filter(|link| {
                if let Some(from_site) = &link.from.site {
                    if let Some(to_site) = &link.to.site {
                        if (from_site.identification.id == client_id
                            || to_site.identification.id == client_id)
                            && (from_site.identification.id == *parent_id
                                || to_site.identification.id == *parent_id)
                        {
                            return true;
                        }
                    }
                }
                false
            })
            .for_each(|link| {
                if link.from.site.is_some()
                    && link.from.site.as_ref().unwrap().identification.id == *parent_id
                {
                    // FROM side is the tower
                    if let Some(ap_device) = devices
                        .iter()
                        .find(|d| link.from.device.is_some() && d.identification.id == link.from.device.as_ref().unwrap().identification.id)
                    {
                        if let Some(name) = ap_device.get_name() {
                            ap_name = Some(name.clone());
                            ap_id = Some(ap_device.get_id());
                        }
                    }
                } else {
                    // TO side is the tower
                    if let Some(ap_device) = devices
                        .iter()
                        .find(|d| link.to.device.is_some() && d.identification.id == link.to.device.as_ref().unwrap().identification.id)
                    {
                        if let Some(name) = ap_device.get_name() {
                            ap_name = Some(name.clone());
                            ap_id = Some(ap_device.get_id());
                        }
                    }
                }
            });
    }

    (ap_name, ap_id)
}

async fn send_unmapped(clients: Vec<String>, url: String) {
    let report = shared_rest::Unmapped { clients };
    let client = reqwest::Client::new();
    let res = client.post(&url).json(&report).send().await;
    if res.is_err() {
        println!("{:?}", res);
    }
}
