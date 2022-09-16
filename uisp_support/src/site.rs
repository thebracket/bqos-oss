use serde::{Deserialize, Serialize};

/// Represents a Site in UISP.
#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct Site {
    /// The Site ID, typically a UUID.
    pub id: String,

    /// The site's identification section.
    pub identification: Option<SiteId>,

    /// The site's description section.
    pub description: Option<Description>,

    /// The site's QoS settings.
    pub qos: Option<Qos>,

    /// The site's additional UCRM information, if present.
    pub ucrm: Option<Ucrm>,
}

impl Site {
    /// Retrieve the name of the site.
    pub fn name(&self) -> Option<String> {
        if let Some(id) = &self.identification {
            if let Some(name) = &id.name {
                return Some(name.clone());
            }
        }
        None
    }

    /// Is this a tower site? True is it is, false if its a client.
    pub fn is_tower(&self) -> bool {
        if let Some(id) = &self.identification {
            if let Some(site_type) = &id.site_type {
                if site_type == "site" {
                    return true;
                }
            }
        }
        false
    }

    /// Is this a client site? True if it is, false if its a tower.
    pub fn is_client_site(&self) -> bool {
        if let Some(id) = &self.identification {
            if let Some(site_type) = &id.site_type {
                if site_type == "endpoint" {
                    return true;
                }
            }
        }
        false
    }

    /// Checks the site's "parent" option to determine if a site is parent of another
    /// site.
    pub fn is_child_of(&self, parent_id: &str) -> bool {
        if let Some(id) = &self.identification {
            if let Some(parent) = &id.parent {
                if let Some(pid) = &parent.id {
                    if pid == parent_id {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Determine the site (client site)'s QoS settings. If it is unspecified in
    /// UISP, it will use the provided defaults.
    pub fn qos(&self, default_download_mbps: u32, default_upload_mbps: u32) -> (u32, u32) {
        let mut down = default_download_mbps;
        let mut up = default_upload_mbps;
        if let Some(qos) = &self.qos {
            if let Some(d) = &qos.downloadSpeed {
                down = *d as u32 / 1_000_000;
            }
            if let Some(u) = &qos.uploadSpeed {
                up = *u as u32 / 1_000_000;
            }
        }
        if down == 0 {
            down = default_download_mbps;
        }
        if up == 0 {
            up = default_upload_mbps;
        }
        (down, up)
    }
}

/// UISP entry for a site's parent.
#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct SiteParent {
    /// The ID of the parent, if specified.
    pub id: Option<String>,
}

/// UISP "identification" section.
#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct SiteId {
    /// The site name, if it exists.
    pub name: Option<String>,
    
    /// What type of site is it? ("endpoint" for clients)
    #[serde(rename = "type")] pub site_type: Option<String>,

    /// The site's parent
    pub parent: Option<SiteParent>,

    /// The site's status
    pub status: Option<String>,

    /// Is the site suspended?
    pub suspended: bool,
}

/// Defines a UISP "endpoint" record.
#[allow(non_snake_case, missing_docs)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Endpoint {
    pub id: Option<String>,
    pub name: Option<String>,
    pub parentId: Option<String>,
}

/// Defines a UISP site description
#[allow(non_snake_case, missing_docs)]
#[derive(Deserialize, Debug)]
pub struct Description {
    pub location: Option<Location>,
    pub height: Option<f64>,
    pub endpoints: Option<Vec<Endpoint>>,
}

/// Defines a site's location (lat/lon)
#[allow(non_snake_case, missing_docs)]
#[derive(Deserialize, Debug)]
pub struct Location {
    pub longitude: f64,
    pub latitude: f64,
}

/// Site QoS definition from UISP
#[allow(non_snake_case, missing_docs)]
#[derive(Deserialize, Debug)]
pub struct Qos {
    pub enabled: bool,
    pub downloadSpeed: Option<usize>,
    pub uploadSpeed: Option<usize>,
}

/// UCRM Linkage information from UISP/NMS.
#[allow(non_snake_case, missing_docs)]
#[derive(Deserialize, Debug)]
pub struct Ucrm {
    pub client: Option<UcrmClient>,
    pub service: Option<UcrmService>,
}

/// UCRM client information.
#[allow(non_snake_case, missing_docs)]
#[derive(Deserialize, Debug)]
pub struct UcrmClient {
    pub id: String,
    pub name: String,
}

/// UCRM Service information
#[allow(non_snake_case, missing_docs)]
#[derive(Deserialize, Debug)]
pub struct UcrmService {
    pub id: String,
    pub name: String,
    pub status: i32,
    pub tariffId: String,
    pub trafficShapingOverrideEnabled: bool,
}
