use super::InfluxQuery;
use anyhow::Result;
use influxdb2::{models::Query, Client};

pub struct InfluxResult<T>(pub Vec<T>);

impl<T: PartialOrd> InfluxResult<T> {
    pub fn sort(mut self) -> Self {
        self.0.sort_by(|a, b| a.partial_cmp(&b).unwrap());
        self
    }
}

fn build_query(query_builder: InfluxQuery) -> String {
    let bucket = &query_builder.bucket;
    let range = &query_builder.range;
    let measurement = &query_builder.measurement;
    let mut query = format!(
        "from(bucket: \"{bucket}\")
    |> range({range})
    |> filter(fn: (r) => r[\"_measurement\"] == \"{measurement}\")
    "
    );
    query_builder
        .filters
        .iter()
        .for_each(|f| query += &f.to_query_string());
    let aggregate_function = query_builder.aggregate_function.name();
    let aggregate = &query_builder.aggregate;
    if let Some(group) = &query_builder.group {
        query += &format!("|> group(columns: [\"{group}\"])\n");
    }
    query += &format!(
        "|> aggregateWindow(every: {aggregate}, fn: {aggregate_function}, createEmpty: false)"
    );

    if query_builder.with_last {
        query += &format!(
            "|> last()
            "
        );
    } else {
        query += &format!(
            "|> yield(name: \"{aggregate_function}\")
        "
        );
    }
    query
}

pub async fn influx_query<T: influxdb2_structmap::FromMap>(
    query_builder: InfluxQuery,
) -> Result<InfluxResult<T>> {
    let cfg = crate::configuration();
    let query = Query::new(build_query(query_builder));

    let client = Client::new(cfg.influx_url, cfg.influx_org, cfg.influx_token);
    let result: Vec<T> = client.query::<T>(Some(query)).await?;

    Ok(InfluxResult(result))
}
