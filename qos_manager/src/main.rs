#[macro_use]
extern crate rocket;
mod bus;
use bus::*;
use std::time::Duration;
mod queries;
use rocket::{fairing::AdHoc, fs::NamedFile};
mod config;
use crate::config::configuration;
pub mod influx;
mod reports;

/// Opens the index page
#[get("/")]
pub async fn index<'a>() -> Option<NamedFile> {
    NamedFile::open("static/index.html").await.ok()
}

/// Opens the system page
#[get("/system")]
pub async fn system<'a>() -> Option<NamedFile> {
    NamedFile::open("static/system.html").await.ok()
}

/// Opens the tree page
#[get("/tree")]
pub async fn tree<'a>() -> Option<NamedFile> {
    NamedFile::open("static/tree.html").await.ok()
}

/// Provides a download for Plotly (graphing library)
#[get("/plotly-2.9.0.min.js")]
pub async fn plotly<'a>() -> Option<NamedFile> {
    NamedFile::open("static/plotly-2.9.0.min.js").await.ok()
}

/// Opens a site page. id is passed but ignored, it is used client-side.
#[get("/site/<_id>")]
pub async fn site<'a>(_id: String) -> Option<NamedFile> {
    NamedFile::open("static/site.html").await.ok()
}

/// Opens an access point page. id is passed but ignored, it is used client-side.
#[get("/access_point/<_id>")]
pub async fn access_point<'a>(_id: String) -> Option<NamedFile> {
    NamedFile::open("static/access_point.html").await.ok()
}

/// Opens a client page. id is passed but ignored, it is used client-side.
#[get("/client/<_id>")]
pub async fn client<'a>(_id: String) -> Option<NamedFile> {
    NamedFile::open("static/client.html").await.ok()
}

/// Opens a list of duplicate IP addresses
#[get("/ip_dupe")]
pub async fn ip_dupe<'a>() -> Option<NamedFile> {
    NamedFile::open("static/ip_dupe.html").await.ok()
}

/// Opens the reports page.
#[get("/reports")]
pub async fn reports_page<'a>() -> Option<NamedFile> {
    NamedFile::open("static/reports.html").await.ok()
}

/// Opens the congestion report
#[get("/congestion")]
pub async fn congestion<'a>() -> Option<NamedFile> {
    NamedFile::open("static/congestion.html").await.ok()
}

/// Opens the AP congestion report
#[get("/congestion_ap")]
pub async fn congestion_ap<'a>() -> Option<NamedFile> {
    NamedFile::open("static/congestion_ap.html").await.ok()
}

/// Opens the client congestion report
#[get("/congestion_client")]
pub async fn congestion_client<'a>() -> Option<NamedFile> {
    NamedFile::open("static/congestion_client.html").await.ok()
}

/// Opens the site latency report
#[get("/site_latency")]
pub async fn site_latency_page<'a>() -> Option<NamedFile> {
    NamedFile::open("static/site_latency.html").await.ok()
}

/// Opens the AP latency report
#[get("/ap_latency")]
pub async fn ap_latency_page<'a>() -> Option<NamedFile> {
    NamedFile::open("static/ap_latency.html").await.ok()
}

/// Opens the client latency report
#[get("/client_latency")]
pub async fn client_latency_page<'a>() -> Option<NamedFile> {
    NamedFile::open("static/client_latency.html").await.ok()
}

/// Opens the page compiling billing plan information
#[get("/billing_plans")]
pub async fn billing_plans_page<'a>() -> Option<NamedFile> {
    NamedFile::open("static/billing_plans.html").await.ok()
}

/// Opens the unknown IP addresses page
#[get("/unknown_ips")]
pub async fn unknown_ip_addresses_page<'a>() -> Option<NamedFile> {
    NamedFile::open("static/unknown_ip.html").await.ok()
}

/// Opens the unmapped clients report
#[get("/unmapped")]
pub async fn unmapped_page<'a>() -> Option<NamedFile> {
    NamedFile::open("static/unmapped.html").await.ok()
}

#[get("/oversell")]
pub async fn oversell_page<'a>() -> Option<NamedFile> {
    NamedFile::open("static/oversell.html").await.ok()
}

#[get("/nightly")]
pub async fn nightly_page<'a>() -> Option<NamedFile> {
    NamedFile::open("static/nightly.html").await.ok()
}

#[get("/bq.js")]
pub async fn bq<'a>() -> Option<NamedFile> {
    NamedFile::open("static/bq.js").await.ok()
}

#[get("/spinner.gif")]
pub async fn spinner<'a>() -> Option<NamedFile> {
    NamedFile::open("static/spinner.gif").await.ok()
}

async fn nightly_reports() {
    reports::nightly_report_runner().await;
}

async fn periodic_uisp_refresh() {
    loop {
        let (_, _, _) = rocket::tokio::join!(
            queries::get_uisp_devices(),
            queries::get_uisp_sites(),
            queries::get_all_crm_service_plans(),
        );
        queries::poll_ap_frequencies().await;
        queries::poll_signals().await;
        println!("Completed UISP Refresh");
        rocket::tokio::time::sleep(Duration::from_secs(300)).await;
    }
}

#[launch]
fn rocket() -> _ {
    config::load_config().unwrap();
    let _ = load_config();
    let _ = load_tree();
    rocket::build()
        .attach(AdHoc::on_liftoff("Get Devices", |_| {
            Box::pin(async move {
                rocket::tokio::spawn(periodic_uisp_refresh());
            })
        }))
        .attach(AdHoc::on_liftoff("Nightly Reports", |_| {
            Box::pin(async move {
                rocket::tokio::spawn(nightly_reports());
            })
        }))
        .mount(
            "/",
            routes![
                bq,
                spinner,
                index,
                system,
                tree,
                plotly,
                site,
                access_point,
                client,
                ip_dupe,
                congestion,
                congestion_ap,
                congestion_client,
                site_latency_page,
                ap_latency_page,
                client_latency_page,
                billing_plans_page,
                unknown_ip_addresses_page,
                unmapped_page,
                host_usage,
                latency_report,
                bandwidth_report,
                duplicate_ip,
                unmapped_clients,
                get_site_config,
                queue_tree,
                add_ap_limit,
                add_site_limit,
                queries::last_cpu_average,
                queries::site_bandwidth,
                queries::latency_site,
                queries::all_tree,
                queries::node_by_id,
                queries::node_by_index,
                queries::node_children,
                queries::site_breadcrumbs,
                queries::last_ram_use,
                queries::last_swap_use,
                queries::post_search,
                queries::peak_bandwidth,
                queries::duplicate_ip,
                queries::unmapped,
                queries::site_funnel,
                queries::site_funnel_sites,
                queries::site_drops,
                queries::find_interface_speed,
                queries::ap_at_10,
                queries::device_at_10,
                queries::ap_frequency,
                queries::ap_noise,
                queries::signal,
                queries::access_point_info,
                queries::site_device_list,
                queries::site_suspended,
                reports_page,
                reports::site_congestion,
                reports::ap_congestion,
                reports::client_congestion,
                reports::site_tcp_latency,
                reports::ap_tcp_latency,
                reports::client_tcp_latency,
                reports::plan_5m,
                reports::billing_plans,
                reports::unknown_ip_addresses,
                reports::oversell_report,
                reports::nightly_json,
                oversell_page,
                nightly_page,
            ],
        )
}
