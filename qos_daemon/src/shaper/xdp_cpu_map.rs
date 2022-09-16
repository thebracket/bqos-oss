use std::process::Stdio;

use crate::pretty::{display_action, display_success};
use anyhow::Result;
use config::QosConfig;
use tokio::process::Command;

/// Issues an `xps_setup.sh` call to disable XPS on an interface.
/// Derived from LibreQOS.
async fn disable_xps(config: &QosConfig, interface: &str) -> Result<()> {
    let xps_setup = format!("{}/bin/xps_setup.sh", &config.xdp_path);
    display_action(&format!("Default XPS for {interface}"), 2);
    // ./xdp-cpumap-tc/bin/xps_setup.sh -d ens19 --default --disable
    Command::new(&xps_setup)
        .arg("-d")
        .arg(interface)
        .arg("--default")
        .arg("--disable")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await?;

    display_success(&format!("Defaulted XPS for {interface}"), 2);

    Ok(())
}

/// Setup the IP Hash system for XDP. Derived from LibreQOS.
async fn xps_ip_hash(config: &QosConfig, interface: &str, lan: bool) -> Result<()> {
    let xdp_iphash_to_cpu = format!("{}/src/xdp_iphash_to_cpu", &config.xdp_path);
    display_action(&format!("Enable XDP Hashing for {interface}"), 2);
    // ./xdp-cpumap-tc/src/xdp_iphash_to_cpu --dev ens19 --lan
    // ./xdp-cpumap-tc/src/xdp_iphash_to_cpu --dev ens20 --wan
    Command::new(&xdp_iphash_to_cpu)
        .arg("--dev")
        .arg(interface)
        .arg(if lan { "--lan" } else { "--wan" })
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await?;

    display_success(&format!("Enabled XDP Hashing for {interface}"), 2);
    Ok(())
}

/// Clear all existing XDP commands. Derived from LibreQOS.
async fn clear_xdp_commands(config: &QosConfig) -> Result<()> {
    let xdp_iphash_to_cpu_cmdline = format!("{}/src/xdp_iphash_to_cpu_cmdline", &config.xdp_path);
    display_action(&format!("Clearing XDP Command List"), 2);
    // ./xdp-cpumap-tc/src/xdp_iphash_to_cpu_cmdline --clear
    Command::new(xdp_iphash_to_cpu_cmdline)
        .arg("--clear")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await?;

    display_success(&format!("Clearing XDP Command List"), 2);
    Ok(())
}

/// Setup the XDP Classification system. Derived from LibreQOS.
async fn xdp_classify(config: &QosConfig, interface: &str) -> Result<()> {
    let tc_classify = format!("{}/src/tc_classify", &config.xdp_path);
    display_action(&format!("Enable Hash Classification for {interface}"), 2);
    // ./xdp-cpumap-tc/src/tc_classify --dev-egress ens19
    Command::new(tc_classify)
        .arg("--dev-egress")
        .arg(interface)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await?;

    display_success(&format!("Enabled Hash Classification for {interface}"), 2);
    Ok(())
}

/// Setup XDP on the ISP and Internet interfaces.
/// Derived directly from LibreQOS's Python code.
pub async fn setup_xdp(config: &QosConfig) -> Result<()> {
    disable_xps(config, &config.to_isp).await?;
    disable_xps(config, &config.to_internet).await?;
    xps_ip_hash(config, &config.to_isp, true).await?;
    xps_ip_hash(config, &config.to_internet, false).await?;
    clear_xdp_commands(config).await?;
    xdp_classify(config, &config.to_isp).await?;
    xdp_classify(config, &config.to_internet).await?;

    Ok(())
}
