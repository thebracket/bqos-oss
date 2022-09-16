use anyhow::Result;
use config::QosConfig;
use lazy_static::*;
use parking_lot::RwLock;
use ron::to_string;
use shared_rest::{ApLimit, ShaperTreeConfig, SiteLimit};
use std::{collections::hash_map::DefaultHasher, hash::Hash};

lazy_static! {
    static ref LIMITS: RwLock<(String, ShaperTreeConfig)> =
        RwLock::new((String::new(), ShaperTreeConfig::new()));
}

fn make_hash(shaper_config: &ShaperTreeConfig) -> String {
    let ron = to_string(shaper_config).unwrap();
    let mut hasher = DefaultHasher::new();
    format!("{:?}", ron.hash(&mut hasher))
}

/// Connects to the `qos_manager` (at /bus/site_config) and downloads
/// a list of all site and AP limits. These are stored in the LIMITS
/// static in two parts:
/// 
/// 0. A hash of the limits result, used to detect changes.
/// 1. The actual list of limits.
pub async fn update_limits(config: &QosConfig) -> Result<()> {
    let full_url = format!("{}/bus/site_config", config.controller_url);
    let client = reqwest::Client::new();

    let res = client
        .get(&full_url)
        .header("'Content-Type", "application/json")
        .send()
        .await?;

    let new_limits = res.json::<ShaperTreeConfig>().await?;
    let new_hash = make_hash(&new_limits);

    let mut lock = LIMITS.write();
    lock.0 = new_hash;
    lock.1 = new_limits;

    crate::display_success("Updated site/ap limits", 2);

    Ok(())
}

pub fn get_limit_hash() -> String {
    LIMITS.read().0.clone()
}

pub fn get_site_limits() -> Vec<SiteLimit> {
    LIMITS.read().1.sites.clone()
}

pub fn get_access_point_limits() -> Vec<ApLimit> {
    LIMITS.read().1.access_points.clone()
}
