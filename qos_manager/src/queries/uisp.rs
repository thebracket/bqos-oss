use crate::{bus::get_queue_tree, config::configuration};
use anyhow::Result;
use config::QosConfig;
use influxdb2::models::DataPoint;
use influxdb2::Client;
use lazy_static::*;
use parking_lot::RwLock;
use rocket::serde::Serialize;
use rocket::tokio::join;
use rocket::{futures::stream, serde::json::Json};
use std::time::Duration;
use uisp_support::crm_types::{ClientServicePlan, ServicePlan};
use uisp_support::{load_all_devices_with_interfaces, load_all_sites_with_crm, Device, Site};

lazy_static! {
    pub static ref DEVICES: RwLock<Vec<Device>> = RwLock::new(Vec::new());
}

lazy_static! {
    pub static ref SITES: RwLock<Vec<Site>> = RwLock::new(Vec::new());
}

lazy_static! {
    pub static ref SERVICE_PLANS: RwLock<Vec<ServicePlan>> = RwLock::new(Vec::new());
}

lazy_static! {
    pub static ref CLIENT_PLANS: RwLock<Vec<ClientServicePlan>> = RwLock::new(Vec::new());
}

lazy_static! {
    pub static ref CLIENTS: RwLock<Vec<uisp_support::crm_types::Client>> = RwLock::new(Vec::new());
}

pub async fn get_uisp_devices() -> Result<()> {
    let mut config = QosConfig::default();
    config.nms_key = configuration().nms_key;
    config.nms_url = configuration().nms_url;
    let devices = load_all_devices_with_interfaces(&config).await?;
    if let Some(mut lock) = DEVICES.try_write_for(Duration::from_secs(2)) {
        *lock = devices;
    }
    Ok(())
}

pub async fn get_uisp_sites() -> Result<()> {
    let mut config = QosConfig::default();
    config.nms_key = configuration().nms_key;
    config.nms_url = configuration().nms_url;
    let sites = load_all_sites_with_crm(&config).await?;
    if let Some(mut lock) = SITES.try_write_for(Duration::from_secs(2)) {
        *lock = sites;
    }
    Ok(())
}

pub async fn get_all_crm_service_plans() -> Result<()> {
    let cfg = configuration();
    let (service_plans, services, clients) = join!(
        uisp_support::get_all_crm_service_plans(&cfg.crm_url, &cfg.crm_key),
        uisp_support::get_all_crm_services(&cfg.crm_url, &cfg.crm_key),
        uisp_support::get_all_crm_clients(&cfg.crm_url, &cfg.crm_key)
    );
    let (service_plans, services, clients) = (service_plans?, services?, clients?);
    if let Some(mut lock) = SERVICE_PLANS.try_write_for(Duration::from_secs(2)) {
        *lock = service_plans;
    }
    if let Some(mut lock) = CLIENT_PLANS.try_write_for(Duration::from_secs(2)) {
        *lock = services;
    }
    if let Some(mut lock) = CLIENTS.try_write_for(Duration::from_secs(2)) {
        *lock = clients;
    }
    Ok(())
}

pub async fn poll_ap_frequencies() {
    let mut tmp = Vec::new();
    {
        let devices = DEVICES.read();
        for ap in get_queue_tree()
            .iter()
            .filter(|t| t.level_type == "ap")
            .map(|t| t.id.clone())
        {
            for device in devices.iter().filter(|d| d.get_id() == ap) {
                if let Some(overview) = &device.overview {
                    if let Some(frequency) = &overview.frequency {
                        tmp.push(
                            DataPoint::builder("frequency")
                                .tag("access_point", &device.get_id())
                                .field("frequency", *frequency)
                                .build()
                                .unwrap(),
                        );
                    }
                }
                if let Some(noise_floor) = device.get_noise_floor() {
                    tmp.push(
                        DataPoint::builder("noise_floor")
                            .tag("access_point", &device.get_id())
                            .field("noise_floor", noise_floor as f64)
                            .build()
                            .unwrap(),
                    );
                }
            }
        }
    }

    let cfg = crate::configuration();
    let client = Client::new(cfg.influx_url, cfg.influx_org, cfg.influx_token);
    let _ = client.write("bracketqos", stream::iter(tmp)).await;
}

pub async fn poll_signals() {
    let mut tmp = Vec::new();
    {
        let devices = DEVICES.read();
        for device in devices.iter() {
            if let Some(overview) = &device.overview {
                if let Some(signal) = &overview.signal {
                    tmp.push(
                        DataPoint::builder("signal")
                            .tag("access_point", &device.get_id())
                            .field("signal", *signal as f64)
                            .build()
                            .unwrap(),
                    )
                }
            }
        }
    }

    let cfg = crate::configuration();
    let client = Client::new(cfg.influx_url, cfg.influx_org, cfg.influx_token);
    let _ = client.write("bracketqos", stream::iter(tmp)).await;
}

fn find_interface_speed_actual(device_id: &str) -> Vec<String> {
    let devices = DEVICES.read();

    let mut speeds = Vec::new();
    devices
        .iter()
        .filter(|d| d.get_id() == device_id)
        .for_each(|device| {
            if let Some(interfaces) = &device.interfaces {
                //println!("{:?}", interfaces);
                for interface in interfaces.iter() {
                    if let Some(status) = &interface.status {
                        if let Some(speed) = &status.speed {
                            speeds.push(format!("{}", speed));
                        }
                    }
                }
            }
        });

    speeds
}

#[get("/query/device_interface_speed/<device_id>")]
pub async fn find_interface_speed(device_id: String) -> Json<Vec<String>> {
    Json(find_interface_speed_actual(&device_id))
}

#[get("/query/10mbit_ap")]
pub async fn ap_at_10() -> Json<Vec<(String, String, String)>> {
    let mut result = Vec::new();
    for t in get_queue_tree().iter().filter(|t| t.level_type == "ap") {
        find_interface_speed_actual(&t.id)
            .iter()
            .filter(|s| *s == "10-full" || *s == "10-half")
            .for_each(|s| {
                //println!("{:?}", (t.id.clone(), t.name.clone(), s.clone()));
                result.push((t.id.clone(), t.name.clone(), s.clone()));
            });
    }
    Json(result)
}

#[get("/query/10mbit_device")]
pub async fn device_at_10() -> Json<Vec<(String, String, String)>> {
    let devices = DEVICES.read();
    let access_points: std::collections::HashSet<String> = get_queue_tree()
        .iter()
        .filter(|t| t.level_type == "ap")
        .map(|t| t.id.clone())
        .collect();
    let mut result = Vec::new();
    for t in devices
        .iter()
        .filter(|d| d.interfaces.is_some() && !access_points.contains(&d.get_id()))
    {
        find_interface_speed_actual(&t.get_id())
            .iter()
            .filter(|s| *s == "10-full" || *s == "10-half")
            .for_each(|s| {
                if let Some(name) = t.get_name() {
                    result.push((t.get_id(), name, s.clone()));
                }
            });
    }
    Json(result)
}

#[derive(Serialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct AccessPointInfo {
    id: String,
    name: String,
    model: String,
    firmware: String,
    status: String,
    frequency: f64,
    outage_score: f64,
    stations_count: i32,
    downlink_capacity: i32,
    uplink_capacity: i32,
    channel_width: i32,
    transmit_power: i32,
    mode: String,
    ip: String,
    noise_floor: i32,
    signal: i32,
    if_speed: Vec<String>,
}

impl AccessPointInfo {
    fn new() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            model: String::new(),
            firmware: String::new(),
            status: String::new(),
            frequency: 0.0,
            outage_score: 1.0,
            stations_count: 0,
            downlink_capacity: 0,
            uplink_capacity: 0,
            channel_width: 0,
            transmit_power: 0,
            mode: String::new(),
            ip: String::new(),
            noise_floor: 0,
            signal: 0,
            if_speed: Vec::new(),
        }
    }
}

async fn access_point_info_actual(id: &str) -> AccessPointInfo {
    let lock = DEVICES.read();
    let mut result = AccessPointInfo::new();
    if let Some(ap) = lock.iter().find(|d| d.get_id() == id) {
        result.id = ap.get_id();
        if let Some(name) = ap.get_name() {
            result.name = name.clone();
        }
        if let Some(model_name) = ap.get_model_name() {
            result.model = model_name.clone();
        }
        if let Some(model) = ap.get_model() {
            result.model += &format!(" ({model})");
        }
        if let Some(fw) = &ap.identification.firmwareVersion {
            result.firmware = fw.clone();
        }
        if let Some(overview) = &ap.overview {
            if let Some(status) = &overview.status {
                result.status = status.clone();
            }
            if let Some(freq) = &overview.frequency {
                result.frequency = *freq;
            }
            if let Some(outage_score) = &overview.outageScore {
                result.outage_score = *outage_score;
            }
            if let Some(stations_count) = &overview.stationsCount {
                result.stations_count = *stations_count;
            }
            if let Some(downlink_capacity) = &overview.downlinkCapacity {
                result.downlink_capacity = *downlink_capacity;
            }
            if let Some(uplink_capacity) = &overview.uplinkCapacity {
                result.uplink_capacity = *uplink_capacity;
            }
            if let Some(channel_width) = &overview.channelWidth {
                result.channel_width = *channel_width;
            }
            if let Some(transmit_power) = &overview.transmitPower {
                result.transmit_power = *transmit_power;
            }
            if let Some(signal) = &overview.signal {
                result.signal = *signal;
            }
        }
        if let Some(mode) = &ap.mode {
            result.mode = mode.clone();
        }
        if let Some(ip) = &ap.ipAddress {
            result.ip = ip.clone();
        }
        if let Some(nf) = ap.get_noise_floor() {
            result.noise_floor = nf;
        }
        result.if_speed = find_interface_speed_actual(id);
    }
    result
}

#[get("/query/access_point_info/<id>")]
pub async fn access_point_info(id: String) -> Json<AccessPointInfo> {
    Json(access_point_info_actual(&id).await)
}

fn find_site_devices(id: &str) -> Vec<String> {
    DEVICES
        .read()
        .iter()
        .filter(|d| {
            if let Some(site) = &d.identification.site {
                if id == site.id {
                    return true;
                }
            }
            return false;
        })
        .map(|d| d.get_id())
        .collect()
}

#[get("/query/site_device_list/<id>")]
pub async fn site_device_list(id: String) -> Json<Vec<AccessPointInfo>> {
    let mut result = Vec::new();
    for site_id in find_site_devices(&id).iter() {
        let device = access_point_info_actual(&site_id).await;
        result.push(device.clone());
    }
    Json(result)
}

#[get("/query/site_suspended/<id>")]
pub async fn site_suspended(id: String) -> Json<bool> {
    if let Some(site) = SITES.read().iter().find(|s| s.id == id) {
        if let Some(site_id) = &site.identification {
            return Json(site_id.suspended);
        }
    }
    return Json(false);
}
