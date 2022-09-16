use lazy_static::*;
use parking_lot::RwLock;
use rocket::serde::json::Json;
use std::time::Duration;

lazy_static! {
    static ref UNMAPPED: RwLock<Vec<String>> = RwLock::new(Vec::new());
}

#[post("/bus/unmapped_clients", data = "<dupes>")]
pub async fn unmapped_clients(dupes: Json<shared_rest::Unmapped>) {
    if let Some(mut lock) = UNMAPPED.try_write_for(Duration::from_secs(2)) {
        *lock = dupes.clients.clone();
    }
}

pub fn unmapped_client_list() -> Vec<String> {
    UNMAPPED.read().clone()
}
