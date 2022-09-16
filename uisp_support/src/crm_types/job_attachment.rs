use serde::Deserialize;

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct JobAttachment {
    pub id: Option<usize>,
    pub size: Option<usize>,
    pub mimeType: Option<String>,
    pub jobId: Option<usize>,
    pub filename: Option<String>,
}
