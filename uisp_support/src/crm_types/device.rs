use serde::Deserialize;

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct Device {
    pub id: Option<usize>,
    pub name: Option<String>,
    pub siteId: Option<usize>,
    pub vendorId: Option<usize>,
    pub modelName: Option<String>,
    pub parentIds: Option<Vec<usize>>,
    pub notes: Option<String>,
    pub loginUsername: Option<String>,
    pub sshPort: Option<usize>,
    pub snmpCommunity: Option<String>,
    pub osVersion: Option<String>,
    pub isGateway: Option<bool>,
    pub isSuspendEnabled: Option<bool>,
    pub sendPingNotification: Option<bool>,
    pub pingNotificationUserId: Option<usize>,
    pub createSignalStatistics: Option<bool>,
}
