use rocket::{futures::stream, serde::json::Json};

#[post("/bus/host", data = "<host>")]
pub async fn host_usage(host: Json<shared_rest::SystemStatus>) {
    use influxdb2::models::DataPoint;
    use influxdb2::Client;

    println!("{}", host.used_memory);

    let mut tmp = vec![
        DataPoint::builder("memory")
            .tag("host", "server01")
            .field(
                "memory",
                (host.used_memory as f64 / host.total_memory as f64) as f64,
            )
            .build()
            .unwrap(),
        DataPoint::builder("swap")
            .tag("host", "server01")
            .field(
                "memory",
                (host.used_swap as f64 / host.total_swap as f64) as f64,
            )
            .build()
            .unwrap(),
    ];
    for (n, usage) in host.cpu_usage.iter().enumerate() {
        tmp.push(
            DataPoint::builder("cpu")
                .tag("host", "server01")
                .tag("cpu", format!("{n}"))
                .field("usage", *usage as f64)
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
