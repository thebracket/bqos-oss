use serde::Deserialize;

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct JobTask {
    pub id: Option<usize>,
    pub jobId: Option<usize>,
    pub label: Option<String>,
}
