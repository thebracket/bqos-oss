#![warn(missing_docs)]

//! Provides definitions for a shared REST API between the web-manager (`qos_manager`)
//! and the QoS control daemon (`qos_daemon`) projects.

mod latency;
pub use latency::*;
mod system;
pub use system::*;
mod bandwidth;
pub use bandwidth::*;
mod duplicate_ips;
pub use duplicate_ips::*;
mod site;
pub use site::*;
mod tree;
pub use tree::*;
mod unmapped;
pub use unmapped::*;
