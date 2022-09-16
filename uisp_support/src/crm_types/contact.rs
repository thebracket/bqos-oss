use serde::Deserialize;

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct Contact {
    pub id: Option<usize>,
    pub clientId: Option<usize>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub name: Option<String>,
    pub isBilling: Option<bool>,
    pub isContact: Option<bool>,
    pub types: Option<Vec<ContactType>>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct ContactType {
    pub id: Option<usize>,
    pub name: Option<String>,
}
