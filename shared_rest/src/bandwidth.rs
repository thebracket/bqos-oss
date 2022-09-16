use serde::{Deserialize, Serialize};

/// Bandwidth usage report. `qos_daemon` periodically gathers this data
/// and sends it to the manager, which handles storing it in InfluxDB
/// and displaying graphs,
#[derive(Serialize, Deserialize, Debug)]
pub struct BandwidthReport {
    /// Timestamp of the bandwidth report
    pub timestamp: String,

    /// List of upload usage
    pub upload: Vec<BandwidthLine>,

    /// List of download usage
    pub download: Vec<BandwidthLine>,
}

/// Line item for a bandwidth report
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BandwidthLine {
    /// Site ID (currently UISP site ID) consuming bandwidth.
    pub site_id: String,

    /// MBPS calculated for the time period.
    pub mbits_per_second: f64,

    /// Number of queue drops in the time period.
    pub drops: u64,
}

impl BandwidthReport {
    /// Create an empty `BandwidthReport` entry with a given
    /// timestamp.
    pub fn new(timestamp: String) -> Self {
        Self {
            timestamp,
            upload: Vec::new(),
            download: Vec::new(),
        }
    }
}
