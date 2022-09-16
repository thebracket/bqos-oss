use serde::{Deserialize, Serialize};

/// API reporting duplicate IPs found within the UISP hierarchy.
#[derive(Serialize, Deserialize, Debug)]
pub struct DuplicateIp {
    /// A list of duplicate IPs.
    pub dupes: Vec<String>,
}
