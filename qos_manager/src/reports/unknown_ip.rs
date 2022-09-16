use crate::bus::UNMAPPED_IP;
use rocket::serde::json::Json;

#[get("/reports/unknown_ip")]
pub async fn unknown_ip_addresses() -> Json<Vec<String>> {
    Json(UNMAPPED_IP.read().iter().map(|(k, _v)| k.clone()).collect())
}

