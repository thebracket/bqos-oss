use serde::Deserialize;

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct JobComment {
    pub id: Option<usize>,
    pub jobId: Option<usize>,
    pub userId: Option<usize>,
    pub createdDate: Option<String>,
    pub message: Option<String>,
}
