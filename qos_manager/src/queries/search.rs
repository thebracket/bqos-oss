use crate::bus::get_queue_tree;
use rocket::serde::{json::Json, Serialize};

#[derive(Serialize, Clone, Debug)]
#[serde(crate = "rocket::serde")]
pub struct SearchResult {
    pub name: String,
    pub id: String,
    pub rtype: String,
}

#[post("/query/search", data = "<term>")]
pub async fn post_search(term: String) -> Json<Vec<SearchResult>> {
    let term = term.to_uppercase();

    let tree = get_queue_tree();
    let result = tree
        .iter()
        .filter(|t| {
            let name = t.name.to_uppercase();
            name.contains(&term)
        })
        .map(|t| SearchResult {
            name: t.name.clone(),
            id: t.id.clone(),
            rtype: t.level_type.clone(),
        })
        .collect();

    Json(result)
}
