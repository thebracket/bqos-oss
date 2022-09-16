mod clear;
use anyhow::Result;
pub use clear::clear_queue_settings;
mod queue_counter;
pub use queue_counter::*;
mod master_queues;
pub use master_queues::*;
mod xdp_cpu_map;
use crate::tree_builder::{QueueTree, QueueType};
use config::QosConfig;
use tokio::task::spawn_blocking;
pub use xdp_cpu_map::setup_xdp;
mod limits;
pub use limits::*;
pub mod tuning;

pub const TC_CMD: &str = "/sbin/tc";

/// Walks the Queue Tree and builds the actual queues in the traffic shaper.
pub async fn build_client_queues(config: &QosConfig, plan: QueueTree) -> Result<()> {
    // Generate each CPU's queue plan independently
    let mut my_plan = plan.clone();
    my_plan.queues.drain(0..).for_each(|cpu_queue| {
        if let QueueType::CpuQueue { cpu_id } = cpu_queue.queue_type {
            //println!("CPU Queue {}", cpu_id);
            let my_config = config.clone();
            spawn_blocking(move || {
                let cpu_id = cpu_id + 1;
                let mut class_id = 5;
                let _ = cpu_queue.walk_and_build(&my_config, cpu_id, 1, &mut class_id);
            });
        }
    });

    Ok(())
}
