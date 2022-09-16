use serde::{Deserialize, Serialize};

/// Defines site and AP bandwidth limits that aren't pulled from UISP.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShaperTreeConfig {
    /// A list of sites
    pub sites: Vec<SiteLimit>,
    /// A list of access points
    pub access_points: Vec<ApLimit>,
}

/// Defines the speed limit for a site
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SiteLimit {
    /// The site ID
    pub id: String,

    /// Download limit (mbps)
    pub download: u32,

    /// Upload limit (mbps)
    pub upload: u32,
}

/// Defines the speed limit of an access point
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApLimit {
    /// Access point device ID
    pub id: String,

    /// Download limit (mbps)
    pub download: u32,

    /// Upload limit (mbps)
    pub upload: u32,
}

impl ShaperTreeConfig {
    /// Create an empty site/AP speed limit list.
    pub fn new() -> Self {
        Self {
            sites: Vec::new(),
            access_points: Vec::new(),
        }
    }
}

impl Default for ShaperTreeConfig {
    fn default() -> Self {
        Self::new()
    }
}
