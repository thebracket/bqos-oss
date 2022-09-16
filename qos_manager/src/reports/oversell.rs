use crate::bus::get_queue_tree;
use rocket::serde::{json::Json, Serialize};
use shared_rest::QueueTreeEntry;

fn sum_bandwidth(tree: &[QueueTreeEntry], node: usize) -> (u32, u32) {
    let (mut down, mut up) = (0, 0);
    if tree[node].level_type == "client" && !tree[node].name.ends_with(" Infrastructure") {
        //println!("{}: {}/{}", tree[node].name, tree[node].down_mbps, tree[node].up_mbps);
        down = tree[node].down_mbps;
        up = tree[node].up_mbps;
    }
    tree.iter()
        .enumerate()
        .filter(|(_, t)| t.parent.unwrap_or(usize::MAX) == node)
        .for_each(|(node_n, _t)| {
            let (cdown, cup) = sum_bandwidth(tree, node_n);
            down += cdown;
            up += cup;
        });
    (down, up)
}

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct OversellNode {
    pub name: String,
    pub id: String,
    pub available: (u32, u32),
    pub sold: (u32, u32),
    pub children: Vec<OversellNode>,
}

impl OversellNode {
    pub fn from_tree(tree: &[QueueTreeEntry], node: usize) -> Self {
        let mut result = Self {
            name: tree[node].name.clone(),
            id: tree[node].id.clone(),
            available: (tree[node].down_mbps, tree[node].up_mbps),
            sold: sum_bandwidth(tree, node),
            children: Vec::new(),
        };
        for (i, _child) in tree
            .iter()
            .enumerate()
            .filter(|(_i, c)| c.level_type != "client" && c.parent.unwrap_or(usize::MAX) == node)
        {
            result.children.push(OversellNode::from_tree(tree, i));
        }
        result.children.sort_by(|a, b| {
            let a_ratio = a.sold.0 as f32 / a.available.0 as f32;
            let b_ratio = b.sold.0 as f32 / b.available.0 as f32;
            b_ratio.partial_cmp(&a_ratio).unwrap()
        });
        result
    }
}

#[get("/reports/oversell")]
pub fn oversell_report() -> Json<OversellNode> {
    let tree = get_queue_tree();
    Json(OversellNode::from_tree(&tree, 0))
}
