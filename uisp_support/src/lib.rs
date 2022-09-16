#![warn(missing_docs)]

//! Provides UISP integration support, using a limited subset of the API --- just enough
//! to load a site/client/data-link map and build a network map.

use self::rest::nms_request_get_vec;
use anyhow::Result;
use config::QosConfig;
mod rest;
mod site;
use crm_types::{Client, ClientServicePlan, ServicePlan};
use rest::{crm_request_get_vec, nms_request_get_one};
pub use site::*;
mod device;
pub use device::*;
mod data_link;
pub use data_link::*;

/// Re-export of all includes CRM/NMS mapping types.
pub mod crm_types;

/// Asynchronously connect to UISP and download a list of all sites, including client sites.
pub async fn load_all_sites(config: &QosConfig) -> Result<Vec<Site>> {
    Ok(nms_request_get_vec("sites", &config.nms_key, &config.nms_url).await?)
}

/// Asynchronously connect to USIP and download a list of all sites, including client sites.
/// Also includes the additional information provided by a uCRM link. This is necessary
/// for retrieving some QoS parameters from UISP.
pub async fn load_all_sites_with_crm(config: &QosConfig) -> Result<Vec<Site>> {
    Ok(nms_request_get_vec("sites?ucrmDetails=true", &config.nms_key, &config.nms_url).await?)
}

/// Asynchronously connect to UISP and download a list of all devices, including interface
/// information.
pub async fn load_all_devices_with_interfaces(config: &QosConfig) -> Result<Vec<Device>> {
    Ok(nms_request_get_vec(
        "devices?withInterfaces=true&authorized=true",
        &config.nms_key,
        &config.nms_url,
    )
    .await?)
}

/// Asynchronously load all data links between sites.
pub async fn load_all_data_links(config: &QosConfig) -> Result<Vec<DataLink>> {
    Ok(nms_request_get_vec("data-links", &config.nms_key, &config.nms_url).await?)
}

/// Asynchronously load a single device, with its interface information.
pub async fn load_device_interfaces(
    config: &QosConfig,
    device_id: &str,
) -> Result<Vec<DeviceInterface>> {
    Ok(nms_request_get_vec(
        &format!("devices/{device_id}/interfaces"),
        &config.nms_key,
        &config.nms_url,
    )
    .await?)
}

/// Asynchronously load a device, without additional interface information.
pub async fn load_device(config: &QosConfig, device_id: &str) -> Result<Device> {
    Ok(nms_request_get_one(
        &format!("devices/{device_id}"),
        &config.nms_key,
        &config.nms_url,
    )
    .await?)
}

/// Asynchronously connect to UISP/CRM and retrieve all service plans.
pub async fn get_all_crm_service_plans(
    api: &str,
    key: &str,
) -> Result<Vec<ServicePlan>, reqwest::Error> {
    let url = "service-plans".to_string();
    Ok(crm_request_get_vec(api, key, &url).await?)
}

/// Asynchronously connect to UISP/CRM and retrieve all services.
pub async fn get_all_crm_services(
    api: &str,
    key: &str,
) -> Result<Vec<ClientServicePlan>, reqwest::Error> {
    let url = "clients/services".to_string();
    Ok(crm_request_get_vec(api, key, &url).await?)
}

/// Asynchronously to USP/CRM and retrieve a list of all clients.
pub async fn get_all_crm_clients(api: &str, key: &str) -> Result<Vec<Client>, reqwest::Error> {
    let url = "clients".to_string();
    Ok(crm_request_get_vec(api, key, &url).await?)
}
