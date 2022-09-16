use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// The `qos_daemon` builds a new "queue tree" whenever a topology change is detected. This is then
/// transmitted to the manager process, allowing it to make a searchable list.
/// 
/// The tree is defined by index within the vector, rather than storing a hierarchy. For example,
/// a `parent` of `Some(0)` is parented from the first entry in the vector.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QueueTreeEntry {
    /// The queue name
    pub name: String,

    /// The UISP queue ID (site, client site, AP, etc.)
    pub id: String,

    /// A string defining level type. Levels are:
    /// * `root` (top of the tree)
    /// * `tower`
    /// * `ap`
    /// * `client`
    /// 
    /// TODO: Make this into an enum!
    pub level_type: String,

    /// Parent in the tree, or `None` if there isn't one. This is an index within the overall vector
    /// of tree entries.
    pub parent: Option<usize>,

    /// The download speed limit of the site (Mbps).
    pub down_mbps: u32,

    /// The upload speed limit of the site (Mbps).
    pub up_mbps: u32,

    /// A set of IP Addresses associated with this site.
    pub ip_addresses: HashSet<String>,
}
