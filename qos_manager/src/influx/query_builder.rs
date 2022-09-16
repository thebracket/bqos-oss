use super::{AggregateFunction, InfluxResult, QueryFilter};
use anyhow::Result;

pub struct InfluxQuery {
    pub bucket: String,
    pub range: String,
    pub aggregate: String,
    pub aggregate_function: AggregateFunction,
    pub measurement: String,
    pub filters: Vec<QueryFilter>,
    pub group: Option<String>,
    pub with_last: bool,
}

impl InfluxQuery {
    pub fn new() -> Self {
        let cfg = crate::configuration();
        Self {
            bucket: cfg.influx_bucket.clone(),
            range: String::new(),
            aggregate: String::new(),
            aggregate_function: AggregateFunction::Last,
            measurement: String::new(),
            filters: Vec::new(),
            group: None,
            with_last: false,
        }
    }

    pub fn with_range_string<T: ToString>(mut self, range: T) -> Self {
        self.range = range.to_string();
        self
    }

    pub fn with_range_from_url(mut self, range: &str) -> Self {
        self.range = urlencoding::decode(range).unwrap().to_string();
        self
    }

    pub fn with_aggregate<T: ToString>(
        mut self,
        aggregate: T,
        aggregate_function: AggregateFunction,
    ) -> Self {
        self.aggregate = aggregate.to_string();
        self.aggregate_function = aggregate_function;
        self
    }

    pub fn with_aggregate_from_url(
        mut self,
        aggregate: &str,
        aggregate_function: AggregateFunction,
    ) -> Self {
        self.aggregate = urlencoding::decode(aggregate).unwrap().to_string();
        self.aggregate_function = aggregate_function;
        self
    }

    pub fn with_measurement<T: ToString>(mut self, measurement: T) -> Self {
        self.measurement = measurement.to_string();
        self
    }

    pub fn with_filter(mut self, filter: QueryFilter) -> Self {
        self.filters.push(filter);
        self
    }

    pub fn with_group<T: ToString>(mut self, group: T) -> Self {
        self.group = Some(group.to_string());
        self
    }

    pub fn with_last(mut self) -> Self {
        self.with_last = true;
        self
    }

    pub async fn run<T: influxdb2_structmap::FromMap>(self) -> Result<InfluxResult<T>> {
        Ok(super::influx_query(self).await?)
    }
}
