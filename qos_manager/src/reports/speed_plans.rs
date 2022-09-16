use crate::bus::get_queue_tree;
use rocket::serde::{json::Json, Serialize};

#[derive(Serialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct SpeedPlan {
    pub id: String,
    pub name: String,
    pub down: u32,
    pub up: u32,
}

#[get("/reports/plan_5m")]
pub async fn plan_5m() -> Json<Vec<SpeedPlan>> {
    let mut result = Vec::new();
    get_queue_tree()
        .iter()
        .filter(|t| t.level_type == "client" && t.down_mbps == 5)
        .for_each(|c| {
            result.push(SpeedPlan {
                id: c.id.clone(),
                name: c.name.clone(),
                down: c.down_mbps,
                up: c.up_mbps,
            });
        });
    Json(result)
}
