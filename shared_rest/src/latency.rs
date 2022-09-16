use serde::{Deserialize, Serialize};
use std::{collections::HashSet, net::Ipv4Addr};

/// API to send min/max/average latency.
/// Currently, only average is sent.
#[derive(Serialize, Deserialize, Debug)]
pub struct MinMaxAvg {
    /// The average latency for the report period for this item.
    pub average: f32,
}

/// A line item in a latency report.
#[derive(Serialize, Deserialize, Debug)]
pub struct LatencyItem {
    /// The site ID (UISP uuid) for which we are reporting
    pub site: String,

    /// A latency average (this used to include min/max as well)
    pub latency: MinMaxAvg,
}

/// A REST latency report, sent periodically.
#[derive(Serialize, Deserialize, Debug)]
pub struct LatencyReport {
    /// Each item included in this report
    pub items: Vec<LatencyItem>,

    /// Detected IP addresses that didn't map to the detected
    /// network topology.
    pub unmapped_ip: Vec<String>,
}

impl LatencyReport {
    /// Create a default latency report, with no entries.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            unmapped_ip: Vec::new(),
        }
    }

    /// Add a line-item to the report.
    pub fn report(&mut self, site: &str, latency: MinMaxAvg) {
        self.items.push(LatencyItem {
            site: site.to_string(),
            latency,
        })
    }

    /// Add an unmapped IP address to the report.
    pub fn add_unmapped(&mut self, unmapped: &HashSet<Ipv4Addr>) {
        self.unmapped_ip
            .extend(unmapped.iter().map(|ip| ip.to_string()));
    }
}

impl Default for LatencyReport {
    fn default() -> Self {
        Self::new()
    }
}
