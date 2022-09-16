use anyhow::Result;
use lazy_static::*;
use parking_lot::RwLock;
use rocket::serde::json::Json;
use ron::ser::{to_string_pretty, PrettyConfig};
use shared_rest::{ApLimit, ShaperTreeConfig, SiteLimit};
use std::time::Duration;

lazy_static! {
    static ref SITE_CONFIG: RwLock<ShaperTreeConfig> = RwLock::new(ShaperTreeConfig::new());
}

#[get("/bus/site_config")]
pub async fn get_site_config() -> Json<ShaperTreeConfig> {
    let lock = SITE_CONFIG.read();
    Json(lock.clone())
}

pub fn load_config() -> Result<()> {
    let data = std::fs::read_to_string("shaper.ron")?;
    let map_file: ShaperTreeConfig = ron::from_str(&data)?;
    if let Some(mut lock) = SITE_CONFIG.try_write_for(Duration::from_secs(2)) {
        *lock = map_file;
    }
    Ok(())
}

fn save_config() -> Result<()> {
    let data = SITE_CONFIG.read().clone();
    let header_ron = to_string_pretty(&data, PrettyConfig::new())?;
    std::fs::write("shaper.ron", header_ron)?;
    Ok(())
}

#[get("/bus/add_site_limit/<id>/<download>/<upload>")]
pub async fn add_site_limit(id: String, download: u32, upload: u32) -> Json<ShaperTreeConfig> {
    if let Some(mut lock) = SITE_CONFIG.try_write_for(Duration::from_secs(2)) {
        if let Some(site) = lock.sites.iter_mut().find(|s| s.id == id) {
            site.download = download;
            site.upload = upload;
        } else {
            lock.sites.push(SiteLimit {
                id,
                download,
                upload,
            })
        }
    }
    let _ = save_config();

    let lock = SITE_CONFIG.read();
    Json(lock.clone())
}

#[get("/bus/add_ap_limit/<id>/<download>/<upload>")]
pub async fn add_ap_limit(id: String, download: u32, upload: u32) -> Json<ShaperTreeConfig> {
    if let Some(mut lock) = SITE_CONFIG.try_write_for(Duration::from_secs(2)) {
        if let Some(ap) = lock.access_points.iter_mut().find(|s| s.id == id) {
            ap.download = download;
            ap.upload = upload;
        } else {
            lock.access_points.push(ApLimit {
                id,
                download,
                upload,
            })
        }
    }
    let _ = save_config();

    let lock = SITE_CONFIG.read();
    Json(lock.clone())
}
