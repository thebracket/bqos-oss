use rocket::{futures::stream, serde::json::Json};

#[post("/bus/bandwidth", data = "<bandwidth>")]
pub async fn bandwidth_report(bandwidth: Json<shared_rest::BandwidthReport>) {
    /*bandwidth
    .download
    .iter()
    .filter(|b| b.mbits_per_second > 0.0)
    .for_each(|b| println!("{:#?}", b));*/

    use influxdb2::models::DataPoint;
    use influxdb2::Client;

    let mut tmp = Vec::new();
    for line in bandwidth.download.iter() {
        tmp.push(
            DataPoint::builder("queues")
                .tag("site", &line.site_id)
                .field("down_mbps", line.mbits_per_second)
                .build()
                .unwrap(),
        );
        tmp.push(
            DataPoint::builder("queues")
                .tag("site", &line.site_id)
                .field("down_drops", line.drops as f64)
                .build()
                .unwrap(),
        );
    }
    for line in bandwidth.upload.iter() {
        tmp.push(
            DataPoint::builder("queues")
                .tag("site", &line.site_id)
                .field("up_mbps", line.mbits_per_second)
                .build()
                .unwrap(),
        );
        tmp.push(
            DataPoint::builder("queues")
                .tag("site", &line.site_id)
                .field("up_drops", line.drops as f64)
                .build()
                .unwrap(),
        );
    }

    let cfg = crate::configuration();
    let client = Client::new(cfg.influx_url, cfg.influx_org, cfg.influx_token);
    let result = client.write("bracketqos", stream::iter(tmp)).await;
    if result.is_err() {
        println!("{:?}", result);
    }
}
