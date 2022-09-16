use crate::{
    shaper::get_site_limits,
    tree_builder::{ip_matchers::ip_addresses_in_site, Queue, QueueTree},
};
use anyhow::Result;
use config::QosConfig;
use uisp_support::{Device, Site};

pub async fn site_only_strategy(
    config: &QosConfig,
    tree: &mut QueueTree,
    sites: &[Site],
    devices: &[Device],
) -> Result<()> {
    let mut top_level_queue = 0;

    let site_limits = get_site_limits();

    // Build top level queues - one per tower
    sites.iter().filter(|s| s.is_tower()).for_each(|site| {
        if let Some(name) = site.name() {
            let (down_mbps, up_mbps) =
                if let Some(limit) = site_limits.iter().find(|sl| sl.id == site.id) {
                    (limit.download, limit.upload)
                } else {
                    (config.internet_download_mbps, config.internet_upload_mbps)
                };

            let mut tower_queue = Queue::new_tower_site(&name, down_mbps, up_mbps, &site.id);

            // Insert infrastructure elements
            if let Ok(ip_addresses) = ip_addresses_in_site(site, devices) {
                for ip in ip_addresses.iter() {
                    tree.ip_to_site_map
                        .insert(ip.clone(), format!("{}.0", site.id));
                }
                let infrastructure = Queue::new_client_site(
                    &format!("{name} Infrastructure"),
                    config.internet_download_mbps,
                    config.internet_upload_mbps,
                    &ip_addresses,
                    &format!("{}.0", site.id),
                );
                tower_queue.children.push(infrastructure);
            }

            // Find client sites with this as the parent
            sites
                .iter()
                .filter(|s| s.is_client_site() && s.is_child_of(&site.id))
                .for_each(|client_site| {
                    if let Some(name) = client_site.name() {
                        if let Ok(ip_addresses) = ip_addresses_in_site(client_site, devices) {
                            if !ip_addresses.is_empty() {
                                let qos = if client_site.is_client_site() {
                                    client_site.qos(
                                        config.default_download_mbps,
                                        config.default_upload_mbps,
                                    )
                                } else {
                                    (config.internet_download_mbps, config.internet_upload_mbps)
                                };
                                for ip in ip_addresses.iter() {
                                    tree.ip_to_site_map
                                        .insert(ip.clone(), client_site.id.clone());
                                }
                                tower_queue.children.push(Queue::new_client_site(
                                    &name,
                                    qos.0,
                                    qos.1,
                                    &ip_addresses,
                                    &client_site.id,
                                ));
                            }
                        }
                    }
                });

            tree.queues[top_level_queue].children.push(tower_queue);

            top_level_queue += 1;
            top_level_queue %=
                u32::min(tree.queue_count.to_isp, tree.queue_count.to_internet) as usize;
        }
    });

    Ok(())
}
