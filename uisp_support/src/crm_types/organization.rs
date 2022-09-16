use serde::Deserialize;

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct Organization {
    pub id: Option<usize>,
    pub name: Option<String>,
    pub registrationNumber: Option<String>,
    pub taxId: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub street1: Option<String>,
    pub street2: Option<String>,
    pub city: Option<String>,
    pub countryId: Option<usize>,
    pub stateId: Option<usize>,
    pub currencyId: Option<usize>,
    pub zipCode: Option<String>,
    pub selected: Option<bool>,
}
