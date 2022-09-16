use crate::pretty::display_action;
use config::QosConfig;
use shared_rest::SystemStatus;
use std::time::Duration;
use sysinfo::{ProcessorExt, SystemExt};
use tokio::spawn;

pub async fn gather_host_info(config: QosConfig) {
    use sysinfo::System;
    let mut sys = System::new_all();
    sys.refresh_all();
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
        display_action("Host Check", 2);
        sys.refresh_all();

        let cpu_usage = sys
            .processors()
            .iter()
            .map(|cpu| cpu.cpu_usage())
            .collect::<Vec<f32>>();
        let report = SystemStatus::new(
            sys.total_memory(),
            sys.used_memory(),
            sys.total_swap(),
            sys.used_swap(),
            cpu_usage,
        );
        // TODO: Send the report
        spawn(send_report(
            report,
            format!("{}/bus/host", &config.controller_url),
        ));
    }
}

async fn send_report(report: SystemStatus, url: String) {
    let client = reqwest::Client::new();
    let _ = client.post(&url).json(&report).send().await;
}
