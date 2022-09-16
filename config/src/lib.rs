#![warn(missing_docs)]
//! The `config` crate handles configuration of the overall QoS system,
//! and serialization/de-serialization of the configuration file.

use anyhow::{Error, Result};
use ron::de::from_reader;
use serde::Deserialize;
use std::{fs::File, path::Path};

/// `ShapingStrategy` defines the method used to build the tree of shaper nodes. It offers three
/// choices:
/// 
/// * `JustClients` - each client gets their own top-level entry. This is the highest performing
/// option, distributing clients evenly across CPU nodes. It doesn't provide any per-backhaul,
/// per-site or per-AP limitation options.
/// * `SiteOnly` - implements a parent queue per site, with all clients of that site placed beneath
/// the site queue. This lets you place absolute limits on each site, but does not provide per-AP or
/// backhaul controls.
/// * `Full` - implements a queue per site, queue per AP. This provides the most control, but
/// performs poorly if large numbers of customers reside beneath a single site.
#[derive(Deserialize, Clone)]
pub enum ShapingStrategy {
    /// Each client gets their own tree entry.
    JustClients, // Useful when you don't have backhaul contention or need to keep server usage down.
    /// Each site gets a tree entry, with clients at that site beneath it.
    SiteOnly,    // One parent queue per site, no hierarchy.
    /// Full Site->AP->Client hierarchy, including site-to-site links (backhauls).
    Full,        // Map everything including site links and access points.
}

/// Defines the configuration to use. Saved in the file `/usr/local/etc/bracket_qos.ron`.
/// *Note*: It is intended that a future release will change this.
#[derive(Deserialize, Clone)]
pub struct QosConfig {
    /// The ISP-facing interface name.
    pub to_isp: String,

    /// The Internet-facing interface name.
    pub to_internet: String,

    /// The path in which the XDP tools reside. I use `/usr/local/xdp-cpumap-tc`.
    pub xdp_path: String,

    /// Total available download on the Internet connection, in Mbps.
    pub internet_download_mbps: u32,

    /// Total available upload on the Internet connection, in Mbps.
    pub internet_upload_mbps: u32,

    /// Default download speed (Mbps) allocated to an unrecognized IP address.
    pub default_download_mbps: u32,

    /// Default upload speed (Mbps) allocated to an unrecognized IP address.
    pub default_upload_mbps: u32,

    /// The UISP/NMS API key to use for obtaining customer data.
    /// *Note*: It is intended to make this optional.
    pub nms_key: String,

    /// The UISP/NMS URL, not including the "/api..." portion.
    /// *Note*: It is intended to make this optional.
    pub nms_url: String,

    /// The name of the tree root site, as it appears in UNMS's site tree.
    /// This is necessary because figuring out the top-level tree can be tricky
    /// if sites aren't correctly parented with data-links in the UNMS
    /// sites graph.
    pub root_site_name: String,

    /// An array of CIDR addresses to include in traffic shaping. For example,
    /// ` [ "172.16.0.0/12", "10.0.0.0/8", "100.64.0.0/10", ]`
    pub include_ip_ranges: Vec<String>,

    /// An array of CIDR addresses to exclude from traffic shaping/tracking.
    pub ignore_ip_ranges: Vec<String>,

    /// The shaping strategy to use (see `ShapingStrategy`, above).
    pub strategy: ShapingStrategy,

    /// The URL of the controller system running `qos_manager`. `qos_daemon` will
    /// periodically forward data to this system. Separating them allows for better
    /// load distribution, and prevents the shaper from wasting CPU time on management
    /// tasks. They *can* be run on the same box.
    pub controller_url: String,
}

/// Where the configuration file is saved
const CONFIG_FILENAME: &str = "/usr/local/etc/bracket_qos.ron";

impl QosConfig {
    /// Loads a site configuration from the path specified in `CONFIG_FILENAME`.
    /// Returns Ok<a configuration> or an error.
    pub fn load() -> Result<Self> {
        let path = Path::new(CONFIG_FILENAME);
        if !path.exists() {
            return Err(Error::msg("Please setup {CONFIG_FILENAME}"));
        }
        let f = File::open(CONFIG_FILENAME)?;
        let mut cfg: Self = from_reader(f)?;
        if cfg.xdp_path.ends_with("/") {
            cfg.xdp_path = cfg.xdp_path[0..cfg.xdp_path.len() - 1].to_string();
        }
        if cfg.nms_url.ends_with('/') {
            cfg.nms_url = format!("{}nms/api/v2.1", cfg.nms_url);
        } else {
            cfg.nms_url = format!("{}/nms/api/v2.1", cfg.nms_url);
        }
        Ok(cfg)
    }
}

impl Default for QosConfig {
    fn default() -> Self {
        Self {
            to_isp: "ens19".to_string(),
            to_internet: "ens20".to_string(),
            xdp_path: "/usr/local/xdp-cpumap-tc".to_string(),
            internet_download_mbps: 1_000,
            internet_upload_mbps: 1_000,
            default_download_mbps: 1_000,
            default_upload_mbps: 1_000,
            nms_key: String::new(),
            nms_url: String::new(),
            root_site_name: String::new(),
            strategy: ShapingStrategy::JustClients,
            include_ip_ranges: Vec::new(),
            ignore_ip_ranges: Vec::new(),
            controller_url: String::new(),
        }
    }
}
