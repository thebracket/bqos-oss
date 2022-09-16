use super::{QueueCount, TC_CMD};
use crate::pretty::{display_action, display_success};
use anyhow::Result;
use config::QosConfig;
use tokio::process::Command;

/// Sets up the 7FFF: master queue (in mq mode) for an each interface.
/// Copied from LibreQOS.
pub async fn set_master_multiqueues(config: &QosConfig) -> Result<()> {
    display_action("Setting Master Queues", 2);
    set_master_multiqueue(&config.to_isp).await?;
    set_master_multiqueue(&config.to_internet).await?;
    Ok(())
}

async fn set_master_multiqueue(interface: &str) -> Result<()> {
    display_action(&format!("Set multiqueue for {}", interface), 3);
    // tc qdisc replace dev ens19 root handle 7FFF: mq
    Command::new(TC_CMD)
        .arg("qdisc")
        .arg("replace")
        .arg("dev")
        .arg(interface)
        .arg("root")
        .arg("handle")
        .arg("7FFF:")
        .arg("mq")
        .status()
        .await?;

    display_success(&format!("Set multiqueue for {}", interface), 3);
    Ok(())
}

/// Builds top-level queues based on queue count for each interface.
/// Derived from LibreQOS.
pub async fn set_master_interface_queues(config: &QosConfig, queues: &QueueCount) -> Result<()> {
    display_action("Setting ISP Facing Queues", 2);
    set_master_queues(
        &config.to_isp,
        queues.to_isp,
        config.internet_download_mbps,
        config.default_download_mbps,
    )
    .await?;
    display_action("Setting Internet Facing Queues", 2);
    set_master_queues(
        &config.to_internet,
        queues.to_internet,
        config.internet_upload_mbps,
        config.default_upload_mbps,
    )
    .await?;
    Ok(())
}

async fn set_master_queues(
    interface: &str,
    n_queues: u32,
    max_mbps: u32,
    defaut_mbps: u32,
) -> Result<()> {
    for queue in 0..n_queues {
        let queue_id = queue + 1;

        Command::new("/sbin/tc")
            .arg("qdisc")
            .arg("add")
            .arg("dev")
            .arg(interface)
            .arg("parent")
            .arg(format!("7FFF:{queue_id}"))
            .arg("handle")
            .arg(format!("{queue_id}:"))
            .arg("htb")
            .arg("default")
            .arg("2")
            .status()
            .await?;

        Command::new("/sbin/tc")
            .arg("class")
            .arg("add")
            .arg("dev")
            .arg(interface)
            .arg("parent")
            .arg(format!("{queue_id}:"))
            .arg("classid")
            .arg(format!("{queue_id}:1"))
            .arg("htb")
            .arg("rate")
            .arg(format!("{max_mbps}mbit"))
            .arg("ceil")
            .arg(format!("{max_mbps}mbit"))
            .status()
            .await?;

        Command::new("/sbin/tc")
            .arg("qdisc")
            .arg("add")
            .arg("dev")
            .arg(interface)
            .arg("parent")
            .arg(format!("{queue_id}:1"))
            .arg("cake")
            .arg("diffserv4")
            .status()
            .await?;

        Command::new("/sbin/tc")
            .arg("class")
            .arg("add")
            .arg("dev")
            .arg(interface)
            .arg("parent")
            .arg(format!("{queue_id}:1"))
            .arg("classid")
            .arg(format!("{queue_id}:2"))
            .arg("htb")
            .arg("rate")
            .arg(format!("{}mbit", defaut_mbps / 4))
            .arg("ceil")
            .arg(format!("{}mbit", defaut_mbps))
            .arg("prio")
            .arg("5")
            .status()
            .await?;

        Command::new("/sbin/tc")
            .arg("qdisc")
            .arg("add")
            .arg("dev")
            .arg(interface)
            .arg("parent")
            .arg(format!("{queue_id}:2"))
            .arg("cake")
            .arg("diffserv4")
            .status()
            .await?;

        display_success(&format!("Parent queue {}:1", queue_id), 3);
    }
    display_success(
        &format!("Set {} master queues for {}", n_queues, interface),
        3,
    );
    Ok(())
}
