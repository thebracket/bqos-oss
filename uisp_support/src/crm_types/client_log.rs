use serde::Deserialize;

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct ClientLog {
    pub id: Option<usize>,
    pub message: Option<String>,
    pub clientId: Option<usize>,
    pub userId: Option<usize>,
    pub createdDate: Option<String>,
}
