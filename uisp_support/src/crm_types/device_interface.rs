use serde::Deserialize;

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct DeviceInterface {
    pub id: Option<usize>,
    pub deviceId: Option<usize>,
    pub ipRanges: Option<Vec<String>>,
    pub name: Option<String>,
    #[serde(alias = "type")]
    pub device_type: Option<usize>,
    pub macAddress: Option<String>,
    pub allowClientConnection: Option<bool>,
    pub enabled: Option<bool>,
    pub notes: Option<String>,
    pub ssid: Option<String>,
    pub frequency: Option<usize>,
    pub polarization: Option<usize>,
    pub encryptionType: Option<usize>,
    pub encryptionKeyWpa: Option<String>,
    pub encryptionKeyWpa2: Option<String>,
}
