use crate::pretty::display_error;
use anyhow::Result;
use cidr::Ipv4Inet;
use config::QosConfig;
use lazy_static::*;
use parking_lot::RwLock;
use std::{net::Ipv4Addr, str::FromStr};
use uisp_support::{Device, Site};

lazy_static! {
    static ref RELEVANT_IPS: RwLock<Vec<Ipv4Inet>> = RwLock::new(Vec::new());
}

lazy_static! {
    static ref IGNORE_IPS: RwLock<Vec<Ipv4Inet>> = RwLock::new(Vec::new());
}

pub fn is_ip_relevant_no_igore(ip: Ipv4Addr) -> bool {
    let relevant_ips = RELEVANT_IPS.read();
    relevant_ips.iter().any(|r| r.contains(&ip))
}

/// Loads the relevant and ignore IPs from a configuration, and load
/// them into a static for quick access. 
pub fn load_ip_matching(config: &QosConfig) {
    let mut relevant_ips = Vec::new();
    let mut ignore_ips = Vec::new();
    for r in config.include_ip_ranges.iter() {
        if let Ok(ipv4) = Ipv4Inet::from_str(&*r) {
            relevant_ips.push(ipv4);
        } else {
            display_error(&format!("Cannot parse {} from Relevant IP Range", r), 3);
        }
    }
    for i in config.ignore_ip_ranges.iter() {
        if let Ok(ipv4) = Ipv4Inet::from_str(&*i) {
            ignore_ips.push(ipv4);
        } else {
            display_error(&format!("Cannot parse {} from Ignore IP Range", i), 3);
        }
    }

    let mut lock = RELEVANT_IPS.write();
    *lock = relevant_ips;

    let mut lock = IGNORE_IPS.write();
    *lock = ignore_ips;
}

pub fn ip_addresses_in_site(client_site: &Site, devices: &[Device]) -> Result<Vec<String>> {
    let relevant_ips = RELEVANT_IPS.read();
    let ignore_ips = IGNORE_IPS.read();

    let site_devices = devices
        .iter()
        .filter(|device| {
            if let Some(site_id) = device.get_site_id() {
                site_id == client_site.id
            } else {
                false
            }
        })
        .collect::<Vec<&Device>>();

    let mut ip_addresses = Vec::new();
    for device in site_devices.iter() {
        for ip in device.get_addresses() {
            if let Ok(ipv4) = Ipv4Addr::from_str(&ip) {
                if ignore_ips.iter().any(|ignore| ignore.contains(&ipv4)) {
                    //println!("Ignoring {}, it is in the ignore subnet ranges", ip);
                } else {
                    if relevant_ips.iter().any(|r| r.contains(&ipv4)) {
                        ip_addresses.push(ip);
                    } else {
                        //println!("Ingoring {}, there's no relevant IP here.", ip);
                    }
                }
            } else {
                return Err(anyhow::Error::msg(format!(
                    "Unable to parse {} into an IPv4",
                    ip
                )));
            }
        }
    }

    Ok(ip_addresses)
}
