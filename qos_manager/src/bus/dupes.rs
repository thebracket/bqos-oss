use lazy_static::*;
use parking_lot::RwLock;
use rocket::serde::json::Json;
use std::time::Duration;

lazy_static! {
    static ref DUPLICATE_IP: RwLock<Vec<String>> = RwLock::new(Vec::new());
}

#[post("/bus/duplicate_ip", data = "<dupes>")]
pub async fn duplicate_ip(dupes: Json<shared_rest::DuplicateIp>) {
    if let Some(mut lock) = DUPLICATE_IP.try_write_for(Duration::from_secs(2)) {
        *lock = dupes.dupes.clone();
    }
}

pub fn duplicate_ip_list() -> Vec<String> {
    DUPLICATE_IP.read().clone()
}
