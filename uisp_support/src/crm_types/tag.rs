use serde::Deserialize;

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct Tag {
    pub id: Option<usize>,
    pub name: Option<String>,
    pub colorBackground: Option<String>,
    pub colorText: Option<String>,
}
