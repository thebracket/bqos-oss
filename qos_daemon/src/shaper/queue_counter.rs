use crate::pretty::display_success;
use anyhow::Result;
use config::QosConfig;
use serde::{Deserialize, Serialize};
use tokio::{fs::read_dir, join};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueCount {
    pub to_isp: u32,
    pub to_internet: u32,
}

/// Collects the number of available upload and download queues.
/// These are separate because it may be desirable in the future
/// to use fewer queues for upload, since it typically has less
/// traffic.
pub async fn count_queues(config: &QosConfig) -> Result<QueueCount> {
    let (to_isp, to_internet) = join!(
        available_queues(&config.to_isp),
        available_queues(&config.to_internet)
    );
    let result = QueueCount {
        to_isp: to_isp?,
        to_internet: to_internet?,
    };
    display_success(
        &format!("Queue Count: {}, {}", result.to_isp, result.to_internet),
        3,
    );
    Ok(result)
}

/// Fetch the number of available queues for an interface from
/// `/sys/class/net/(interface)/queues`.
/// If we're in a hypervisor, limit to a maximum of 9.
async fn available_queues(interface: &str) -> Result<u32> {
    let path = format!("/sys/class/net/{interface}/queues/");
    let mut reader = read_dir(&path).await?;
    let mut queue_count = 0;
    while let Some(f) = reader.next_entry().await? {
        if f.path().to_str().unwrap().contains("tx-") {
            queue_count += 1;
        }
    }
    if are_we_in_a_hypervisor().await? {
        // Recommendation not to use more than 9 queues in a VM.
        queue_count = u32::min(queue_count, 9);
    }
    Ok(queue_count)
}

/// Checks "/proc/cpuinfo" to see if we're in a hypervisor.
async fn are_we_in_a_hypervisor() -> Result<bool> {
    // Read /proc/cpuinfo
    let cpu_info = tokio::fs::read_to_string("/proc/cpuinfo").await?;
    Ok(cpu_info.contains("hypervisor"))
}
