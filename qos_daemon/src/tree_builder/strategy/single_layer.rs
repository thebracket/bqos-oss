use crate::{
    pretty::display_success,
    tree_builder::{ip_matchers::ip_addresses_in_site, Queue, QueueTree},
};
use anyhow::Result;
use config::QosConfig;
use uisp_support::{Device, Site};

pub async fn single_layer_strategy(
    config: &QosConfig,
    tree: &mut QueueTree,
    sites: &[Site],
    devices: &[Device],
) -> Result<()> {
    let mut top_level_queue = 0;

    sites
        .iter()
        //.filter(|s| s.is_client_site())
        .for_each(|client_site| {
            if let Some(name) = client_site.name() {
                if let Ok(ip_addresses) = ip_addresses_in_site(client_site, devices) {
                    if !ip_addresses.is_empty() {
                        let qos = if client_site.is_client_site() {
                            client_site
                                .qos(config.default_download_mbps, config.default_upload_mbps)
                        } else {
                            (config.internet_download_mbps, config.internet_upload_mbps)
                        };
                        for ip in ip_addresses.iter() {
                            tree.ip_to_site_map
                                .insert(ip.clone(), client_site.id.clone());
                        }
                        tree.queues[top_level_queue]
                            .children
                            .push(Queue::new_client_site(
                                &name,
                                qos.0,
                                qos.1,
                                &ip_addresses,
                                &client_site.id,
                            ));

                        top_level_queue += 1;
                        top_level_queue %=
                            u32::min(tree.queue_count.to_isp, tree.queue_count.to_internet)
                                as usize;
                    }
                }
            }
        });

    display_success(&format!("Mapped {} IPs", tree.ip_to_site_map.len()), 3);
    //println!("{:#?}", tree.queues);
    Ok(())
}
