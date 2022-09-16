use super::{site_bandwidth_query, InternetBandwidthRest};
use crate::bus::get_tree_children;
use rocket::{
    futures::future::join_all,
    serde::{json::Json, Serialize},
};

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct InternetBandwidthFunnel {
    pub sites: Vec<(String, Vec<InternetBandwidthRest>)>,
}

#[get("/query/site_funnel/<id>/<range>/<aggregate>")]
pub async fn site_funnel(
    id: String,
    range: String,
    aggregate: String,
) -> Json<InternetBandwidthFunnel> {
    let mut result = InternetBandwidthFunnel { sites: Vec::new() };
    for child in get_tree_children(&id).iter() {
        result
            .sites
            .push(site_funnel_data(child.id.clone(), &range, &aggregate, child.name.clone()).await);
    }

    result.sites.sort_by(|a, b| {
        let a_max = a.1.iter().map(|n| n.down).reduce(f64::max).unwrap();
        let b_max = b.1.iter().map(|n| n.down).reduce(f64::max).unwrap();
        a_max.partial_cmp(&b_max).unwrap()
    });

    Json(result)
}

#[get("/query/site_funnel_sites/<id>/<range>/<aggregate>")]
pub async fn site_funnel_sites(
    id: String,
    range: String,
    aggregate: String,
) -> Json<InternetBandwidthFunnel> {
    let mut tasks = Vec::new();
    let mut children: Vec<(String, String)> = get_tree_children(&id)
        .iter()
        .map(|child| (child.id.clone(), child.name.clone()))
        .collect();

    children.drain(0..).for_each(|(cid, cname)| {
        let cname = if cname.len() > 10 {
            format!("{}...", cname[0..10].to_string())
        } else {
            cname
        };
        tasks.push(site_funnel_data(cid, &range, &aggregate, cname));
    });

    let mut results = join_all(tasks).await;
    let mut result = InternetBandwidthFunnel { sites: Vec::new() };
    results.drain(0..).for_each(|r| result.sites.push(r));

    result.sites.sort_by(|a, b| {
        let a_max = a.1.iter().map(|n| n.down).reduce(f64::max).unwrap();
        let b_max = b.1.iter().map(|n| n.down).reduce(f64::max).unwrap();
        b_max.partial_cmp(&a_max).unwrap()
    });

    const MAX_SITES: usize = 16;

    if result.sites.len() > MAX_SITES + 1 {
        let mut others = result.sites[5].1.clone();
        result.sites.remove(MAX_SITES);
        while result.sites.len() > MAX_SITES + 1 {
            result.sites[5].1.iter().enumerate().for_each(|(i, stats)| {
                if others.len() > i {
                    others[i].down += stats.down;
                }
                if others.len() > i {
                    others[i].up += stats.up;
                }
            });
            result.sites.remove(MAX_SITES);
        }
        result.sites.push(("Others".to_string(), others));
    }

    Json(result)
}

async fn site_funnel_data(
    id: String,
    range: &str,
    aggregate: &str,
    name: String,
) -> (String, Vec<InternetBandwidthRest>) {
    (
        name.to_string(),
        site_bandwidth_query(&id, range, aggregate).await.unwrap(),
    )
}
