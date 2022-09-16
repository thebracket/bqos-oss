use serde::Deserialize;

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct InvoiceTemplate {
    pub id: Option<usize>,
    pub name: Option<usize>,
    pub createdDate: Option<String>,
    pub isOfficial: Option<bool>,
    pub isValid: Option<bool>,
}
