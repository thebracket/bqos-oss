use rocket::serde::json::Json;
use shared_rest::QueueTreeEntry;

use crate::bus::{get_queue_tree, get_tree_children, get_tree_node_by_id, get_tree_node_by_index};

#[get("/query/all_tree")]
pub async fn all_tree() -> Json<Vec<QueueTreeEntry>> {
    Json(get_queue_tree())
}

#[get("/query/node/<id>")]
pub async fn node_by_id(id: String) -> Json<QueueTreeEntry> {
    Json(get_tree_node_by_id(&id))
}

#[get("/query/node_index/<index>")]
pub async fn node_by_index(index: usize) -> Json<QueueTreeEntry> {
    Json(get_tree_node_by_index(index))
}

#[get("/query/children/<id>")]
pub async fn node_children(id: String) -> Json<Vec<QueueTreeEntry>> {
    let mut children = get_tree_children(&id);
    children.sort_by(|a, b| a.name.cmp(&b.name));
    Json(children)
}

#[get("/query/site_crumbs/<id>")]
pub async fn site_breadcrumbs(id: String) -> Json<Vec<QueueTreeEntry>> {
    let mut result = Vec::new();

    let mut current_node = get_tree_node_by_id(&id);
    result.push(current_node.clone());

    while let Some(parent) = current_node.parent {
        current_node = get_tree_node_by_index(parent);
        result.push(current_node.clone());
    }

    Json(result)
}
