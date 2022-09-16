use crate::bus::{duplicate_ip_list, unmapped_client_list};
use rocket::serde::json::Json;

#[get("/query/ip_dupe")]
pub async fn duplicate_ip() -> Json<Vec<String>> {
    Json(duplicate_ip_list())
}

#[get("/query/unmapped")]
pub async fn unmapped() -> Json<Vec<String>> {
    Json(unmapped_client_list())
}
