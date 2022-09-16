use serde::Deserialize;

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct Attribute {
    pub id: Option<usize>,
    pub clientId: Option<usize>,
    pub name: Option<String>,
    pub key: Option<String>,
    pub clientZoneVisible: Option<bool>,
    pub value: Option<String>,
    pub customAttributeId: Option<usize>,
}
