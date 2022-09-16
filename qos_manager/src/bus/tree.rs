use anyhow::Result;
use lazy_static::*;
use parking_lot::RwLock;
use rocket::serde::json::Json;
use ron::ser::{to_string_pretty, PrettyConfig};
use shared_rest::QueueTreeEntry;
use std::time::Duration;

lazy_static! {
    static ref QUEUE_TREE: RwLock<Vec<QueueTreeEntry>> = RwLock::new(Vec::new());
}

#[post("/bus/tree", data = "<tree>")]
pub async fn queue_tree(tree: Json<Vec<QueueTreeEntry>>) {
    //println!("{:?}", tree);
    if let Some(mut lock) = QUEUE_TREE.try_write_for(Duration::from_secs(2)) {
        *lock = tree.to_vec();
    }
    let _ = save_tree();
}

pub fn get_queue_tree() -> Vec<QueueTreeEntry> {
    QUEUE_TREE.read().clone()
}

pub fn get_tree_node_by_id(id: &str) -> QueueTreeEntry {
    QUEUE_TREE
        .read()
        .iter()
        .find(|t| t.id == id)
        .cloned()
        .unwrap()
}

pub fn get_tree_node_by_index(id: usize) -> QueueTreeEntry {
    QUEUE_TREE.read().iter().nth(id).cloned().unwrap()
}

pub fn get_tree_children(id: &str) -> Vec<QueueTreeEntry> {
    let tree = QUEUE_TREE.read();
    tree.iter()
        .filter(|t| {
            if let Some(parent) = t.parent {
                if tree[parent].id == id {
                    return true;
                }
            }
            false
        })
        .cloned()
        .collect()
}

pub fn load_tree() -> Result<()> {
    let data = std::fs::read_to_string("tree.ron")?;
    let map_file: Vec<QueueTreeEntry> = ron::from_str(&data)?;
    if let Some(mut lock) = QUEUE_TREE.try_write_for(Duration::from_secs(2)) {
        *lock = map_file;
    }
    Ok(())
}

fn save_tree() -> Result<()> {
    let data = QUEUE_TREE.read().clone();
    let header_ron = to_string_pretty(&data, PrettyConfig::new())?;
    std::fs::write("tree.ron", header_ron)?;
    Ok(())
}

pub fn get_parent_ids(start: &str) -> Vec<String> {
    let mut current = get_tree_node_by_id(start);
    let mut result = vec![current.id.clone()];
    while current.parent.is_some() {
        current = get_tree_node_by_index(current.parent.unwrap());
        result.push(current.id);
    }
    result
}
