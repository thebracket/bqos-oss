use crate::bus::{get_queue_tree, get_tree_node_by_index};
use crate::queries::{CLIENTS, CLIENT_PLANS, SERVICE_PLANS, SITES};
use rocket::serde::{json::Json, Serialize};

#[derive(Serialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct BillingInfo {
    pub id: String,
    pub name: String,
    pub down: u32,
    pub up: u32,
    pub parent_id: String,
    pub parent_name: String,
    pub site_id: String,
    pub site_name: String,
    pub crm_site: String,
    pub suspended: String,
    pub price: f32,
    pub reseller: String,
    pub outstanding: f32,
}

#[get("/reports/billing_plans")]
pub async fn billing_plans() -> Json<Vec<BillingInfo>> {
    let mut result = Vec::new();
    get_queue_tree()
        .iter()
        .filter(|t| t.level_type == "client")
        .filter(|c| !c.name.contains(" Infrastructure"))
        .for_each(|c| {
            let mut plan_name = String::new();
            let mut suspended = String::new();
            let mut price = 0.0;
            let mut reseller = String::new();
            let mut outstanding = 0.0;
            let crm_site = SITES
                .read()
                .iter()
                .filter(|s| s.id == c.id)
                .filter_map(|s| {
                    if let Some(crm) = &s.ucrm {
                        if let Some(site) = &crm.service {
                            return Some(site.id.clone());
                        }
                    }
                    None
                })
                .next();
            if let Some(crm_site) = crm_site {
                let site_id = crm_site.as_str().parse::<usize>().unwrap_or(0);
                CLIENT_PLANS
                    .read()
                    .iter()
                    .filter(|p| p.id.unwrap_or(0) == site_id)
                    .for_each(|p| {
                        let service_plan_id = p.servicePlanId.unwrap_or(0);
                        SERVICE_PLANS
                            .read()
                            .iter()
                            .filter(|sp| sp.id == service_plan_id)
                            .for_each(|sp| {
                                plan_name = sp.name.clone().unwrap_or(String::new());
                                if let Some(org) = sp.organizationId {
                                    match org {
                                        2 => reseller = "Quantum".to_string(),
                                        3 => reseller = "Tranquility".to_string(),
                                        _ => reseller = "iZones".to_string(),
                                    }
                                }

                                let mut divisor = 1.0;
                                sp.periods.iter().for_each(|p| {
                                    if p.price.is_some() && p.period.is_some() {
                                        divisor = p.period.unwrap() as f32;
                                    }
                                });
                                price = p.price.unwrap_or(0.0) as f32 / divisor;

                                let client_id = SITES
                                    .read()
                                    .iter()
                                    .filter(|s| s.id == c.id)
                                    .filter_map(|s| {
                                        if let Some(crm) = &s.ucrm {
                                            if let Some(client) = &crm.client {
                                                return Some(client.id.clone());
                                            }
                                        }
                                        None
                                    })
                                    .next();

                                if price == 0.0 {
                                    if let Some(client_id) = &client_id {
                                        let client_id =
                                            client_id.as_str().parse::<usize>().unwrap_or(0);
                                        CLIENTS
                                            .read()
                                            .iter()
                                            .filter(|c| c.id.unwrap_or(0) == client_id)
                                            .for_each(|client| {
                                                if let Some(attr) = &client.attributes {
                                                    for a in attr.iter() {
                                                        if let Some(name) = &a.name {
                                                            if name == "Reseller Charge" {
                                                                price = a
                                                                    .value
                                                                    .as_ref()
                                                                    .unwrap()
                                                                    .parse::<f32>()
                                                                    .unwrap_or(0.0);
                                                            }
                                                        }
                                                    }
                                                }
                                            });
                                    }
                                }

                                if let Some(status) = &p.status {
                                    match status {
                                        0 => suspended = "Prepared".to_string(),
                                        1 => suspended = "Active".to_string(),
                                        2 => suspended = "Ended".to_string(),
                                        3 => suspended = "Suspended".to_string(),
                                        4 => suspended = "Prepared blocked".to_string(),
                                        5 => suspended = "Obsolete".to_string(),
                                        6 => suspended = "Deferred".to_string(),
                                        7 => suspended = "Quoted".to_string(),
                                        _ => suspended = "Inactive".to_string(),
                                    }
                                }

                                if let Some(client_id) = client_id {
                                    let client_id =
                                        client_id.as_str().parse::<usize>().unwrap_or(0);
                                    CLIENTS
                                        .read()
                                        .iter()
                                        .filter(|c| c.id.unwrap_or(0) == client_id)
                                        .for_each(|client| {
                                            outstanding = client.accountBalance.unwrap_or(0.0);
                                        });
                                }
                            })
                    });
            }
            let parent = get_tree_node_by_index(c.parent.unwrap_or(0));
            let site = get_tree_node_by_index(parent.parent.unwrap_or(0));
            result.push(BillingInfo {
                id: c.id.clone(),
                name: c.name.clone(),
                down: c.down_mbps,
                up: c.up_mbps,
                parent_id: parent.id.clone(),
                parent_name: parent.name.clone(),
                crm_site: plan_name,
                suspended,
                price,
                reseller,
                site_id: site.id.clone(),
                site_name: site.name.clone(),
                outstanding,
            });
        });
    result.sort_by_key(|k| (k.site_name.clone(), k.parent_name.clone(), k.name.clone()));
    Json(result)
}
