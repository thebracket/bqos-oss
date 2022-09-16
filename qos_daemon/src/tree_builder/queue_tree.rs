use crate::{
    graphing::map_ip_to_site,
    shaper::{count_queues, QueueCount, TC_CMD},
};
use anyhow::Result;
use config::QosConfig;
use ron::{
    ser::{to_string_pretty, PrettyConfig},
    to_string,
};
use serde::{Deserialize, Serialize};
use shared_rest::QueueTreeEntry;
use std::{
    collections::{hash_map::DefaultHasher, HashMap, HashSet},
    hash::Hash,
    process::{Command, Stdio},
};

const LAST_KNOWN_GOOD: &str = "/usr/local/etc/last_known_good_tree.ron";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueTree {
    pub queue_count: QueueCount,
    pub ip_to_site_map: HashMap<String, String>,
    pub queues: Vec<Queue>,
}

impl QueueTree {
    /// Creates an empty queue tree, with one entry per logical CPU we
    /// have available.
    pub async fn new(config: &QosConfig) -> Result<Self> {
        // Create one queue per CPU
        let queue_count = count_queues(config).await?;
        let mut queues = Vec::new();
        for i in 0..u32::min(queue_count.to_internet, queue_count.to_isp) {
            queues.push(Queue::new_cpu_queue(i + 1));
        }

        Ok(Self {
            queue_count,
            ip_to_site_map: HashMap::new(),
            queues,
        })
    }

    /// Attempt to load a queue tree from `/usr/local/etc/last_known_good_tree.ron`
    /// This just de-serializes (via RON/Serde) a file.
    pub fn from_last_known_good() -> Result<Self> {
        let path = std::path::Path::new(LAST_KNOWN_GOOD);
        if !path.exists() {
            return Err(anyhow::Error::msg("No known-good plan found."));
        }
        let f = std::fs::File::open(LAST_KNOWN_GOOD)?;
        let cfg: Self = ron::de::from_reader(f)?;
        Ok(cfg)
    }

    /// Serialize a queue tree to `/usr/local/etc/last_known_good_tree.ron`
    pub fn save_last_good_tree(&self) -> Result<()> {
        let tree_ron = to_string_pretty(&self, PrettyConfig::new())?;
        std::fs::write(LAST_KNOWN_GOOD, tree_ron)?;
        Ok(())
    }

    /// Create a hash of a queue tree, for change detection.
    pub fn make_hash(&self) -> String {
        let ron = to_string(&self.queues).unwrap();
        let mut hasher = DefaultHasher::new();
        format!("{:?}", ron.hash(&mut hasher))
    }

    /// Converts a QueueTree to a `QueueTreeEntry` vector, in the format
    /// required by the REST API.
    pub fn to_monitor_tree(&self, config: &QosConfig) -> Vec<QueueTreeEntry> {
        let mut result = Vec::new();
        result.push(QueueTreeEntry {
            name: "Root".to_string(),
            id: "root".to_string(),
            level_type: "root".to_string(),
            parent: None,
            down_mbps: config.internet_download_mbps,
            up_mbps: config.internet_upload_mbps,
            ip_addresses: HashSet::new(),
        });

        let n_queues = u32::min(self.queue_count.to_internet, self.queue_count.to_isp);
        for c in self.queues.iter().take(n_queues as usize) {
            c.to_tree(0, &mut result);
        }

        result
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueueType {
    CpuQueue {
        cpu_id: u32,
    },
    ClientSite {
        site_id: String,
        down_mbps: u32,
        up_mbps: u32,
        ip_addresses: HashSet<String>,
    },
    TowerSite {
        site_id: String,
        down_mbps: u32,
        up_mbps: u32,
    },
    AccessPointSite {
        site_id: String,
        down_mbps: u32,
        up_mbps: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Queue {
    pub name: String, // Not actually used, but makes for better debug decoration
    pub queue_type: QueueType,
    pub children: Vec<Queue>,
}

impl Queue {
    pub fn new_cpu_queue(cpu_id: u32) -> Self {
        Self {
            name: format!("CPU {cpu_id} Queue"),
            queue_type: QueueType::CpuQueue { cpu_id },
            children: Vec::new(),
        }
    }

    pub fn new_client_site(
        name: &str,
        down_mbps: u32,
        up_mbps: u32,
        ip_addresses: &[String],
        site_id: &str,
    ) -> Self {
        Self {
            name: format!("{name}"),
            queue_type: QueueType::ClientSite {
                site_id: site_id.to_string(),
                down_mbps,
                up_mbps,
                ip_addresses: ip_addresses.iter().cloned().collect(),
            },
            children: Vec::new(),
        }
    }

    pub fn new_tower_site(name: &str, down_mbps: u32, up_mbps: u32, site_id: &str) -> Self {
        Self {
            name: format!("{name}"),
            queue_type: QueueType::TowerSite {
                site_id: site_id.to_string(),
                down_mbps,
                up_mbps,
            },
            children: Vec::new(),
        }
    }

    pub fn new_access_point_site(name: &str, down_mbps: u32, up_mbps: u32, site_id: &str) -> Self {
        Self {
            name: format!("{name}"),
            queue_type: QueueType::AccessPointSite {
                site_id: site_id.to_string(),
                down_mbps,
                up_mbps,
            },
            children: Vec::new(),
        }
    }

    pub fn walk_and_build(
        &self,
        config: &QosConfig,
        cpu_id: u32,
        minor_parent: u32,
        class_id: &mut u32,
    ) -> Result<()> {
        // Build client sites
        match &self.queue_type {
            QueueType::CpuQueue { cpu_id } => {
                //println!("CPU {}", cpu_id);
                // We've already built the CPU queue, so we don't have to make it.
                // We do need to walk the children
                if !self.children.is_empty() {
                    for c in self.children.iter() {
                        c.walk_and_build(config, *cpu_id, 1, class_id)?;
                    }
                }
            }
            QueueType::TowerSite {
                site_id,
                down_mbps,
                up_mbps,
            } => {
                //println!("Tower {}, {}", self.name, class_id);
                // Build the HTB queue for the site
                add_tower_htb(&config.to_isp, cpu_id, minor_parent, *class_id, *down_mbps)?;
                add_tower_htb(
                    &config.to_internet,
                    cpu_id,
                    minor_parent,
                    *class_id,
                    *up_mbps,
                )?;
                crate::graphing::map_htb_queue_to_site((cpu_id, *class_id), site_id);
                let new_minor_parent = *class_id;
                *class_id += 1;

                // Walk children, passing the parent ID
                //println!("Walking {} children", self.children.len());
                if !self.children.is_empty() {
                    for c in self.children.iter() {
                        let result = c.walk_and_build(config, cpu_id, new_minor_parent, class_id);
                        if result.is_err() {
                            println!("{:?}", result);
                        }
                    }
                }
            }
            QueueType::AccessPointSite {
                site_id,
                down_mbps,
                up_mbps,
            } => {
                //println!("AP {}, {}:{}", self.name, cpu_id, class_id);
                // Build the HTB queue for the site
                add_tower_htb(&config.to_isp, cpu_id, minor_parent, *class_id, *down_mbps)?;
                add_tower_htb(
                    &config.to_internet,
                    cpu_id,
                    minor_parent,
                    *class_id,
                    *up_mbps,
                )?;
                crate::graphing::map_htb_queue_to_site((cpu_id, *class_id), site_id);
                let new_minor_parent = *class_id;
                *class_id += 1;

                // Walk children, passing the parent ID
                //println!("Walking {} children", self.children.len());
                if !self.children.is_empty() {
                    for c in self.children.iter() {
                        let result = c.walk_and_build(config, cpu_id, new_minor_parent, class_id);
                        if result.is_err() {
                            println!("{:?}", result);
                        }
                    }
                }
            }
            QueueType::ClientSite {
                site_id,
                down_mbps,
                up_mbps,
                ip_addresses,
            } => {
                //println!("Client {}, {}", self.name, class_id);
                // Client Sites aren't allowed children, so we don't walk them
                // Build a top-level queue for the client, and a child-queue that represents the Cake
                // map. Also add IP hashes.
                add_client_site_htb(&config.to_isp, cpu_id, minor_parent, *class_id, *down_mbps)?;
                crate::graphing::map_queue_to_site((cpu_id, *class_id), site_id);
                add_client_cake(&config.to_isp, cpu_id, *class_id)?;
                add_client_site_htb(
                    &config.to_internet,
                    cpu_id,
                    minor_parent,
                    *class_id,
                    *up_mbps,
                )?;
                add_client_cake(&config.to_internet, cpu_id, *class_id)?;
                let xdp_iphash_to_cpu =
                    format!("{}/src/xdp_iphash_to_cpu_cmdline", &config.xdp_path);
                for ip in ip_addresses.iter() {
                    map_ip_to_site(ip, &site_id);
                    add_queue_hash(&xdp_iphash_to_cpu, ip, cpu_id, *class_id)?;
                }
                let new_minor_parent = *class_id;
                *class_id += 1;

                // Walk children, passing the parent ID
                //println!("Walking {} children", self.children.len());
                if !self.children.is_empty() {
                    for c in self.children.iter() {
                        let result = c.walk_and_build(config, cpu_id, new_minor_parent, class_id);
                        if result.is_err() {
                            println!("{:?}", result);
                        }
                    }
                }
            }
        } // end match

        Ok(())
    }

    fn to_tree(&self, parent: usize, tree: &mut Vec<QueueTreeEntry>) {
        let mut new_parent = parent;
        match &self.queue_type {
            QueueType::CpuQueue { .. } => {}
            QueueType::TowerSite {
                site_id,
                down_mbps,
                up_mbps,
            } => {
                new_parent = tree.len();
                tree.push(QueueTreeEntry {
                    name: self.name.clone(),
                    id: site_id.clone(),
                    level_type: "tower".to_string(),
                    parent: Some(parent),
                    down_mbps: *down_mbps,
                    up_mbps: *up_mbps,
                    ip_addresses: HashSet::new(),
                });
            }
            QueueType::AccessPointSite {
                site_id,
                down_mbps,
                up_mbps,
            } => {
                new_parent = tree.len();
                tree.push(QueueTreeEntry {
                    name: self.name.clone(),
                    id: site_id.clone(),
                    level_type: "ap".to_string(),
                    parent: Some(parent),
                    down_mbps: *down_mbps,
                    up_mbps: *up_mbps,
                    ip_addresses: HashSet::new(),
                });
            }
            QueueType::ClientSite {
                site_id,
                down_mbps,
                up_mbps,
                ip_addresses,
                ..
            } => {
                new_parent = tree.len();
                tree.push(QueueTreeEntry {
                    name: self.name.clone(),
                    id: site_id.clone(),
                    level_type: "client".to_string(),
                    parent: Some(parent),
                    down_mbps: *down_mbps,
                    up_mbps: *up_mbps,
                    ip_addresses: ip_addresses.clone(),
                });
            }
        }

        for c in self.children.iter() {
            c.to_tree(new_parent, tree);
        }
    }
}

fn add_client_site_htb(
    interface: &str,
    cpu_id: u32,
    minor_parent: u32,
    class_id: u32,
    mbps: u32,
) -> Result<()> {
    //println!("tc class add dev {interface} parent {cpu_id}:{minor_parent} classid {class_id}...");
    Command::new(TC_CMD)
        .arg("class")
        .arg("add")
        .arg("dev")
        .arg(interface)
        .arg("parent")
        .arg(&format!("{cpu_id}:{minor_parent}"))
        .arg("classid")
        .arg(&format!("{cpu_id}:{class_id}"))
        .arg("htb")
        .arg("rate")
        .arg(&format!("{}mbit", (mbps as f32 / 2.0).ceil() as u32))
        .arg("ceil")
        .arg(&format!("{}mbit", ((mbps as f32) * 1.09).ceil()))
        .arg("prio")
        .arg("3")
        .status()?;
    Ok(())
}

fn add_client_cake(interface: &str, cpu_id: u32, class_id: u32) -> Result<()> {
    Command::new(TC_CMD)
        .arg("qdisc")
        .arg("add")
        .arg("dev")
        .arg(interface)
        .arg("parent")
        .arg(&format!("{cpu_id}:{}", class_id))
        .arg("cake")
        .arg("diffserv4")
        .status()?;
    Ok(())
}

fn add_queue_hash(xdp_iphash_to_cpu: &str, ip: &str, cpu_id: u32, class_id: u32) -> Result<()> {
    Command::new(&xdp_iphash_to_cpu)
        .arg("--add")
        .arg("--ip")
        .arg(ip)
        .arg("--cpu")
        .arg(&format!("{}", cpu_id - 1))
        .arg("--classid")
        .arg(&format!("{cpu_id}:{}", class_id))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    Ok(())
}

fn add_tower_htb(
    interface: &str,
    cpu_id: u32,
    minor_parent: u32,
    class_id: u32,
    mbps: u32,
) -> Result<()> {
    //println!("tc class add dev {interface} parent {cpu_id}:{minor_parent} classid {class_id} htb rate {mbps}mbit ceil {mbps}mbit prio 3");
    //shell('tc class add dev ' + interfaceA + ' parent ' + parentClassID + ' classid ' + str(minor) + ' htb rate '+ str(round(elemDownloadMin)) + 'mbit ceil '+ str(round(elemDownloadMax)) + 'mbit prio 3')
    //shell('tc class add dev ' + interfaceB + ' parent ' + parentClassID + ' classid ' + str(minor) + ' htb rate '+ str(round(elemUploadMin)) + 'mbit ceil '+ str(round(elemUploadMax)) + 'mbit prio 3')
    Command::new(TC_CMD)
        .arg("class")
        .arg("add")
        .arg("dev")
        .arg(interface)
        .arg("parent")
        .arg(&format!("{cpu_id}:{minor_parent}"))
        .arg("classid")
        .arg(&format!("{cpu_id}:{class_id}"))
        .arg("htb")
        .arg("rate")
        .arg(&format!("{}mbit", (mbps as f32 * 0.95) as u32))
        .arg("ceil")
        .arg(&format!("{}mbit", mbps))
        .arg("prio")
        .arg("3")
        .status()?;

    Ok(())
}
