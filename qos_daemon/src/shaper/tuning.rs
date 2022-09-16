use anyhow::*;
use config::QosConfig;
use tokio::process::Command;
use crate::pretty::display_success;

/// Performs interface tuning:
/// * Disables TCP offloading on each interface.
/// * Disables VLAN offloading on each interface.
/// * Enables BPF JIT.
pub async fn interface_tuning(config: &QosConfig) -> Result<()> {
    no_offload(&config.to_internet).await?;
    no_offload(&config.to_isp).await?;
    no_vlan_offload(&config.to_internet).await?;
    no_vlan_offload(&config.to_isp).await?;
    jit_bpf().await?;
    //htb_rate_fix().await?;
    Ok(())
}

async fn no_offload(interface: &str) -> Result<()> {
    Command::new("/sbin/ethtool")
        .arg("--offload")
        .arg(interface)
        .arg("gso")
        .arg("off")
        .arg("tso")
        .arg("off")
        .arg("lro")
        .arg("off")
        .arg("sg")
        .arg("off")
        .arg("gro")
        .arg("off")
        .status()
        .await?;

    display_success(&format!("Disabled hardware offloading on {}", interface), 2);

    Ok(())
}

async fn no_vlan_offload(interface: &str) -> Result<()> {
    Command::new("/sbin/ethtool")
        .arg("-K")
        .arg(interface)
        .arg("rxvlan")
        .arg("off")
        .status()
        .await?;

    display_success(&format!("Disabled VLAN offloading on {}", interface), 2);

    Ok(())
}

async fn jit_bpf() -> Result<()> {
    Command::new("/sbin/sysctl")
        .arg("net.core.bpf_jit_enable=1")
        .status()
        .await?;

    display_success("Enabled BPF JIT", 2);

    Ok(())
}

// This was removed because it gives wildly inaccurate data sometimes.
/*#[deprecated]
async fn htb_rate_fix() -> Result<()> {
    Command::new("/sbin/modprobe")
        .arg("sch_htb")
        .status()
        .await?;

    let data = "1";
    fs::write("/sys/module/sch_htb/parameters/htb_rate_est", data).await?;
    display_success("Enabled HTB Rate Estimation", 2);
    Ok(())
}*/
