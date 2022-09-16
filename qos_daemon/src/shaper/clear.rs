use super::TC_CMD;
use crate::pretty::{display_action, display_success};
use anyhow::Result;
use config::QosConfig;
use tokio::process::Command;

async fn clear_filter_device(interface: &str) -> Result<()> {
    display_action(&format!("Clearing filter for Device {interface}"), 3);
    Command::new(TC_CMD)
        .arg("filter")
        .arg("delete")
        .arg("dev")
        .arg(interface)
        .status()
        .await?;

    display_success(&format!("Cleared filter for Device {interface}"), 3);
    Ok(())
}

async fn clear_filter_device_root(interface: &str) -> Result<()> {
    display_action(&format!("Clearing filter root for Device {interface}"), 3);
    Command::new(TC_CMD)
        .arg("filter")
        .arg("delete")
        .arg("dev")
        .arg(interface)
        .arg("root")
        .status()
        .await?;

    display_success(&format!("Cleared filter root for Device {interface}"), 3);
    Ok(())
}

async fn clear_qdisc_device_root(interface: &str) -> Result<()> {
    display_action(&format!("Clearing qdisc root for Device {interface}"), 3);
    Command::new(TC_CMD)
        .arg("qdisc")
        .arg("delete")
        .arg("dev")
        .arg(interface)
        .arg("root")
        .status()
        .await?;

    display_success(&format!("Cleared qdisc root for Device {interface}"), 3);
    Ok(())
}

async fn clear_qdisc_device(interface: &str) -> Result<()> {
    display_action(&format!("Clearing qdisc for Device {interface}"), 3);
    Command::new(TC_CMD)
        .arg("qdisc")
        .arg("delete")
        .arg("dev")
        .arg(interface)
        .status()
        .await?;

    display_success(&format!("Cleared qdisc for Device {interface}"), 3);
    Ok(())
}

/// Clears all Linux interface queues for the ISP and Internet interfaces.
/// Derived from LibreQOS.
pub async fn clear_queue_settings(config: &QosConfig) -> Result<()> {
    display_action("Clearing Prior Queue Settings", 2);
    clear_filter_device(&config.to_isp).await?;
    clear_filter_device_root(&config.to_isp).await?;
    clear_qdisc_device_root(&config.to_isp).await?;
    clear_qdisc_device(&config.to_isp).await?;

    clear_filter_device(&config.to_internet).await?;
    clear_filter_device_root(&config.to_internet).await?;
    clear_qdisc_device_root(&config.to_internet).await?;
    clear_qdisc_device(&config.to_internet).await?;
    display_success("Cleared Prior QOS Settings", 2);
    Ok(())
}
