use super::JobAttachment;
use super::JobTask;
use serde::Deserialize;

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct Job {
    pub id: Option<usize>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub assignedUserId: Option<usize>,
    pub clientId: Option<usize>,
    pub date: Option<String>,
    pub duration: Option<f32>,
    pub status: Option<usize>,
    pub address: Option<String>,
    pub gpsLat: Option<f32>,
    pub gpsLon: Option<f32>,
    pub attachments: Option<Vec<JobAttachment>>,
    pub tasks: Option<Vec<JobTask>>,
}
