use serde::{Deserialize, Serialize};

/// `qos_daemon` periodically gathers system status, and uses this to send it
/// to the manager.
#[derive(Serialize, Deserialize, Debug)]
pub struct SystemStatus {
    /// Total memory installed
    pub total_memory: u64,

    /// Total memory in use
    pub used_memory: u64,

    /// Total swap installed
    pub total_swap: u64,

    /// Total swap in use
    pub used_swap: u64,

    /// Vector of CPU usage, by CPU
    pub cpu_usage: Vec<f32>,
}

impl SystemStatus {
    /// Constructor for system status
    pub fn new(
        total_memory: u64,
        used_memory: u64,
        total_swap: u64,
        used_swap: u64,
        cpu_usage: Vec<f32>,
    ) -> Self {
        Self {
            total_memory,
            used_memory,
            total_swap,
            used_swap,
            cpu_usage,
        }
    }
}
