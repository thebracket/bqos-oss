//! Builds CAKE QoS trees, in a hierarchy. A queue tree is built from UISP, according to the
//! defined strategy. Trees are then built, and statistics polled and sent to the manager.
//! UISP is periodically polled, and if the configuration has changed the tree is rebuilt.

use anyhow::Result;
use std::time::Duration;
use tree_builder::QueueTree;
mod pretty;
mod shaper;
mod tree_builder;
mod version;
use pretty::*;
use shaper::{get_limit_hash, update_limits};
use tokio::join;
mod graphing;

#[tokio::main]
async fn main() -> Result<()> {
    display_version();

    // Load the configuration (currently hard-coded to `/usr/local/etc/bracket_qos.ron`)
    // Crash if no configuration could be loaded.
    display_action("Loading Configuration", 1);
    let config = config::QosConfig::load()?;

    // Initialize the IP matching system with the IP include/ignore
    // lists from the configuration file.
    tree_builder::load_ip_matching(&config);

    // Run the "interface tuning" code. Disables TCP offloading,
    // VLAN offloading (which breaks reading shaped data) and
    // enables BPF JIT for a tiny performance improvement.
    display_action("Interface Tuning", 1);
    shaper::tuning::interface_tuning(&config).await?;

    // The `qos_manager` stores per-AP and per-site speed limits.
    // Request these from the manager process via a REST request.
    // We're ignoring any errors, so the system will start correctly
    // even if the manager isn't running. Don't worry, we'll keep trying.
    display_action("Fetching Limits from Controller", 1);
    let _ = update_limits(&config).await; // Ignoring error

    // Try to build a queue plan (`QueueTree`). If no plan can be built, try to
    // load the last functional configuration from `/usr/local/etc/last_known_good_tree.ron`
    // (a serialized dump of the last successful tree build).
    // If it still can't build a tree, it crashes - rather than perform undefined
    // behavior. This will preserve any previous tree structure.
    let queue_plan = if let Ok(plan) = build_plan(&config).await {
        plan
    } else {
        QueueTree::from_last_known_good()?
    };

    // Perform the basic XDP/XPS setup.
    display_action("XPS Interface Setup", 1);
    shaper::setup_xdp(&config).await?;

    // Clear all existing QoS config.
    display_action("Clearing QoS Config", 1);
    shaper::clear_queue_settings(&config).await?;

    // Sets up master multiqueue modes and master interface queues.
    // Copied from LibreQOS.
    display_action("Setting Interface Queues", 1);
    shaper::set_master_multiqueues(&config).await?;
    let queue_count = shaper::count_queues(&config).await?;
    shaper::set_master_interface_queues(&config, &queue_count).await?;

    // Build the actual queues
    display_action("Building Initial Queues", 1);
    let plan_hash = queue_plan.make_hash(); // Hash the queue list for change detection
    shaper::build_client_queues(&config, queue_plan).await?;

    // Create a Future for each long-running task:
    // * Checking UISP for updates.
    // * Polling interface statistics
    // * Polling latency gathering
    // * Host information
    //
    // Then join! on them to run them concurrently. They are designed to run
    // forever...
    display_action("Polling for Changes & Graph Updates", 1);
    let updater = check_for_updates(plan_hash, &config);
    let interface_poller = graphing::gather_interface_stats(&config);
    let latency = graphing::gather_latency(&config);
    let host_info = graphing::gather_host_info(config.clone());
    let _ = join!(updater, interface_poller, latency, host_info);

    // So we never actually get here unless things have gone wrong.
    Ok(())
}

/// Connects to UISP and downloads network information.
/// It then applies the strategy from the configuration to build a `QueueTree`, representing
/// all of the queues to create.
async fn build_plan(config: &config::QosConfig) -> Result<tree_builder::QueueTree> {
    // Concurrently download sites, devices and data-links from UISP.
    display_action("Loading UISP Data", 1);
    let sites_future = uisp_support::load_all_sites(&config);
    let devices_future = uisp_support::load_all_devices_with_interfaces(&config);
    let data_links_future = uisp_support::load_all_data_links(&config);
    let (sites, devices, data_links) = join!(sites_future, devices_future, data_links_future);

    // Unwrap the results. If any of these fail, it will return an Error from the
    // function.
    let sites = sites?;
    let devices = devices?;
    let data_links = data_links?;

    // Build the logical tree based on the downloaded data
    display_action("Building Logical Shaper Tree", 1);
    let tree = tree_builder::build_logical_tree(&config, &sites, &devices, &data_links).await?;

    // Try to save the last-known-good setup. Ignore errors, failure here isn't critical.
    let _ = tree.save_last_good_tree();
    Ok(tree)
}

/// Periodically re-downloads the queue tree from UISP (without applying it). Hash it,
/// and compare the hash to the previous version. If it has changed, then we tear down
/// the queues and re-apply the new scheme - notifying the host.
async fn check_for_updates(previous_hash: String, config: &config::QosConfig) -> Result<()> {
    let mut last_hash = previous_hash;
    let mut last_limit = get_limit_hash();
    loop {
        // Wait for 5 minutes
        tokio::time::sleep(Duration::from_secs(300)).await;

        // Try to build a new plan
        let queue_plan = build_plan(config).await;

        // Update the Qos Manager limits if possible. If they haven't changed,
        // it will keep using the previous limits.
        let _ = update_limits(&config).await; // Ignoring error

        // If we got a new plan - apply it and let the manager know
        if let Ok(queue_plan) = queue_plan {
            // TODO: Update site config and also rebuild if it changed
            let plan_hash = queue_plan.make_hash();
            let limit_hash = get_limit_hash();
            if plan_hash == last_hash && last_limit == limit_hash {
                display_action("No Changes Detected", 1);
            } else {
                last_limit = limit_hash;
                let queue_count = shaper::count_queues(&config).await?;
                last_hash = plan_hash;
                display_action("XPS Interface Setup", 1);
                shaper::setup_xdp(&config).await?;

                display_action("Clearing QoS Config", 1);
                shaper::clear_queue_settings(&config).await?;

                display_action("Setting Interface Queues", 1);
                shaper::set_master_multiqueues(&config).await?;
                shaper::set_master_interface_queues(&config, &queue_count).await?;

                display_action("Building Initial Queues", 1);
                shaper::build_client_queues(&config, queue_plan).await?;
            }
        }
    }
    //Ok(())
}
