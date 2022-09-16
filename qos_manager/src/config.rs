use anyhow::{Error, Result};
use lazy_static::*;
use parking_lot::RwLock;
use rocket::serde::{Deserialize, Serialize};
use ron::de::from_reader;
use std::{fs::File, path::Path, time::Duration};

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct QosManagerConfig {
    pub influx_url: String,
    pub influx_org: String,
    pub influx_token: String,
    pub influx_bucket: String,
    pub nms_key: String,
    pub nms_url: String,
    pub crm_key: String,
    pub crm_url: String,
}

impl QosManagerConfig {
    fn default() -> Self {
        Self {
            influx_url: String::new(),
            influx_org: String::new(),
            influx_token: String::new(),
            influx_bucket: String::new(),
            nms_key: String::new(),
            nms_url: String::new(),
            crm_key: String::new(),
            crm_url: String::new(),
        }
    }
}

lazy_static! {
    static ref CONFIG: RwLock<QosManagerConfig> = RwLock::new(QosManagerConfig::default());
}

const CONFIG_FILENAME: &str = "qos_manager.ron";

pub fn load_config() -> Result<()> {
    let path = Path::new(CONFIG_FILENAME);
    if !path.exists() {
        return Err(Error::msg("Please setup {CONFIG_FILENAME}"));
    }
    let f = File::open(CONFIG_FILENAME).unwrap();
    let cfg: QosManagerConfig = from_reader(f)?;
    if let Some(mut lock) = CONFIG.try_write_for(Duration::from_secs(2)) {
        *lock = cfg;
    }
    Ok(())
}

pub fn configuration() -> QosManagerConfig {
    CONFIG.read().clone()
}
