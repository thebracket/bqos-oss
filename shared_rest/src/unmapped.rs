use serde::{Deserialize, Serialize};

/// A list of clients that failed to be mapped within the hierarchy.
/// These wind up added to a special "unmapped" site.
/// You tend to get these if you haven't defined data-links between sites.
#[derive(Serialize, Deserialize, Debug)]
pub struct Unmapped {
    /// A list of client names.
    pub clients: Vec<String>,
}
