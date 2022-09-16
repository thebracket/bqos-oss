use super::peak_latency;
use crate::bus::get_queue_tree;
use anyhow::{Error, Result};
use rocket::{
    serde::{json::Json, Deserialize, Serialize},
    tokio::{join, time::sleep},
};
use ron::{
    de::from_reader,
    ser::{to_string_pretty, PrettyConfig},
};
use std::{fs::File, path::Path, time::Duration};

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct NightlyReport {
    pub reports: Vec<(String, ApReport, SiteReport)>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct ApReport {
    pub access_points: Vec<(String, f64)>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct SiteReport {
    pub sites: Vec<(String, f64)>,
}

pub async fn nightly_report_runner() {
    // 2 minute pause to ensure things are loaded
    //sleep(Duration::from_secs(120)).await;
    loop {
        let (ap, site) = join!(ap_report(), site_report());

        let mut report = if let Ok(report) = load_nightly() {
            report
        } else {
            NightlyReport {
                reports: Vec::new(),
            }
        };

        if let Ok(ap) = ap {
            if let Ok(site) = site {
                let now = chrono::offset::Local::now().to_rfc3339();
                report.reports.push((now, ap, site));
            }
        }

        report.reports.sort_by(|a, b| a.0.cmp(&b.0));
        while report.reports.len() > 30 {
            report.reports.remove(0);
        }
        let _ = save_nightly(report);

        sleep(Duration::from_secs(86400)).await;
    }
}

const NIGHTLY_FILE: &str = "nightly.ron";

fn load_nightly() -> Result<NightlyReport> {
    let path = Path::new(NIGHTLY_FILE);
    if !path.exists() {
        return Err(Error::msg("Please setup {CONFIG_FILENAME}"));
    }
    let f = File::open(NIGHTLY_FILE).unwrap();
    let cfg: NightlyReport = from_reader(f)?;
    Ok(cfg)
}

fn save_nightly(nightly: NightlyReport) -> Result<()> {
    let header_ron = to_string_pretty(&nightly, PrettyConfig::new())?;
    std::fs::write(NIGHTLY_FILE, header_ron)?;
    Ok(())
}

async fn ap_report() -> Result<ApReport> {
    let mut report = ApReport {
        access_points: Vec::new(),
    };
    for t in get_queue_tree().iter().filter(|t| t.level_type == "ap") {
        let median_latency = peak_latency(t.id.clone()).await;
        report.access_points.push((t.name.clone(), median_latency));
    }
    Ok(report)
}

async fn site_report() -> Result<SiteReport> {
    let mut report = SiteReport { sites: Vec::new() };
    for t in get_queue_tree().iter().filter(|t| t.level_type == "tower") {
        let median_latency = peak_latency(t.id.clone()).await;
        report.sites.push((t.name.clone(), median_latency));
    }
    Ok(report)
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct NightlyCount {
    pub good: u32,
    pub medium: u32,
    pub bad: u32,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct NightlyCountReport {
    pub sites: Vec<(String, NightlyCount)>,
    pub access_points: Vec<(String, NightlyCount)>,
}

#[get("/reports/nightly")]
pub fn nightly_json() -> Json<NightlyCountReport> {
    let n = load_nightly().unwrap();
    let mut result = NightlyCountReport {
        sites: Vec::new(),
        access_points: Vec::new(),
    };
    for (date, ap, site) in n.reports.iter() {
        let mut ap_count = NightlyCount {
            good: 0,
            medium: 0,
            bad: 0,
        };
        let mut site_count = NightlyCount {
            good: 0,
            medium: 0,
            bad: 0,
        };
        for (_name, latency) in ap.access_points.iter() {
            if *latency < 100.0 {
                ap_count.good += 1;
            } else if *latency < 150.0 {
                ap_count.medium += 1;
            } else {
                ap_count.bad += 1;
            }
        }
        for (_name, latency) in site.sites.iter() {
            if *latency < 100.0 {
                site_count.good += 1;
            } else if *latency < 150.0 {
                site_count.medium += 1;
            } else {
                site_count.bad += 1;
            }
        }
        result.access_points.push((date.clone(), ap_count));
        result.sites.push((date.clone(), site_count));
    }
    Json(result)
}
