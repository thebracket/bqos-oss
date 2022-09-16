use std::collections::HashSet;
use serde::Deserialize;

/// UISP Device Link
#[allow(non_snake_case, missing_docs)]
#[derive(Deserialize, Debug)]
pub struct Device {
    pub identification: DeviceIdentification,
    pub ipAddress: Option<String>,
    pub attributes: Option<DeviceAttributes>,
    pub mode: Option<String>,
    pub interfaces: Option<Vec<DeviceInterface>>,
    pub overview: Option<DeviceOverview>,
}

impl Device {
    /// Retrieves the display name of a device
    pub fn get_name(&self) -> Option<String> {
        if let Some(hostname) = &self.identification.hostname {
            return Some(hostname.clone());
        }
        None
    }

    /// Retrieves the model of a device
    pub fn get_model(&self) -> Option<String> {
        if let Some(model) = &self.identification.model {
            return Some(model.clone());
        }
        None
    }

    /// Retrieves the device model's "display name", which may be user-entered.
    pub fn get_model_name(&self) -> Option<String> {
        if let Some(model) = &self.identification.modelName {
            return Some(model.clone());
        }
        None
    }

    /// Retreives the device ID (typically a UUID)
    pub fn get_id(&self) -> String {
        self.identification.id.clone()
    }

    /// Retreives the site ID in which a device resides.
    pub fn get_site_id(&self) -> Option<String> {
        if let Some(site) = &self.identification.site {
            return Some(site.id.clone());
        }
        None
    }

    fn strip_ip(ip: &String) -> String {
        if !ip.contains("/") {
            ip.clone()
        } else {
            ip[0..ip.find("/").unwrap()].to_string()
        }
    }

    /// Build a list of IP addresses for all interfaces attached to a device.
    pub fn get_addresses(&self) -> HashSet<String> {
        let mut result = HashSet::new();
        if let Some(ip) = &self.ipAddress {
            result.insert(Device::strip_ip(ip));
        }
        if let Some(interfaces) = &self.interfaces {
            for interface in interfaces {
                if let Some(addresses) = &interface.addresses {
                    for addy in addresses {
                        if let Some(cidr) = &addy.cidr {
                            result.insert(Device::strip_ip(cidr));
                        }
                    }
                }
            }
        }
        result
    }

    /// Retrieve the Noise Floor from UISP.
    pub fn get_noise_floor(&self) -> Option<i32> {
        if let Some(interfaces) = &self.interfaces {
            for intf in interfaces.iter() {
                if let Some(w) = &intf.wireless {
                    if let Some(nf) = &w.noiseFloor {
                        return Some(*nf);
                    }
                }
            }
        }
        None
    }
}

/// UISP Device Identification section.
#[allow(non_snake_case, missing_docs)]
#[derive(Deserialize, Debug)]
pub struct DeviceIdentification {
    pub id: String,
    pub hostname: Option<String>,
    pub mac: Option<String>,
    pub model: Option<String>,
    pub modelName: Option<String>,
    pub role: Option<String>,
    pub site: Option<DeviceSite>,
    pub firmwareVersion: Option<String>,
}

/// UISP Device Site information
#[allow(non_snake_case, missing_docs)]
#[derive(Deserialize, Debug)]
pub struct DeviceSite {
    pub id: String,
    pub parent: Option<DeviceParent>,
}

/// UISP Device Parent information.
/// This is useful for finding the parent in an AP-Station relationship.
#[allow(non_snake_case, missing_docs)]
#[derive(Deserialize, Debug)]
pub struct DeviceParent {
    pub id: String,
    pub name: String,
}

/// UISP device attributes. Used to find SSID and Access Point.
/// On earlier UISP versions, "parent" was reliable. On later versions,
/// this works.
#[allow(non_snake_case, missing_docs)]
#[derive(Deserialize, Debug)]
pub struct DeviceAttributes {
    pub ssid: Option<String>,
    pub apDevice: Option<DeviceAccessPoint>,
}

/// UISP Access Point linkage.
#[allow(non_snake_case, missing_docs)]
#[derive(Deserialize, Debug)]
pub struct DeviceAccessPoint {
    pub id: Option<String>,
    pub name: Option<String>,
}

/// UISP Device Interface definition
#[allow(non_snake_case, missing_docs)]
#[derive(Deserialize, Debug)]
pub struct DeviceInterface {
    pub identification: Option<InterfaceIdentification>,
    pub addresses: Option<Vec<DeviceAddress>>,
    pub status: Option<InterfaceStatus>,
    pub wireless: Option<DeviceWireless>,
}

/// UISP Interface Identification
#[allow(non_snake_case, missing_docs)]
#[derive(Deserialize, Debug)]
pub struct InterfaceIdentification {
    pub name: Option<String>,
    pub mac: Option<String>,
}

/// UISP IP Address for Device entry.
#[allow(non_snake_case, missing_docs)]
#[derive(Deserialize, Debug)]
pub struct DeviceAddress {
    pub cidr: Option<String>,
}

/// UISP Interface status. Used for 10mbps detection in the manager.
#[allow(non_snake_case, missing_docs)]
#[derive(Deserialize, Debug)]
pub struct InterfaceStatus {
    pub status: Option<String>,
    pub speed: Option<String>,
}

/// UISP Device Overview. You need a recent version for this to be
/// any use at all.
#[allow(non_snake_case, missing_docs)]
#[derive(Deserialize, Debug)]
pub struct DeviceOverview {
    pub status: Option<String>,
    pub frequency: Option<f64>,
    pub outageScore: Option<f64>,
    pub stationsCount: Option<i32>,
    pub downlinkCapacity: Option<i32>,
    pub uplinkCapacity: Option<i32>,
    pub channelWidth: Option<i32>,
    pub transmitPower: Option<i32>,
    pub signal: Option<i32>,
}

/// UISP Wireless status link. Used to retrieve the noise floor.
#[allow(non_snake_case, missing_docs)]
#[derive(Deserialize, Debug)]
pub struct DeviceWireless {
    pub noiseFloor: Option<i32>,
}
