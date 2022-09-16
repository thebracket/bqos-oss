pub enum AggregateFunction {
    Min,
    Max,
    Last,
    Median,
    Mean,
}

impl AggregateFunction {
    pub fn name(&self) -> String {
        match self {
            AggregateFunction::Last => "last".to_string(),
            AggregateFunction::Max => "max".to_string(),
            AggregateFunction::Min => "min".to_string(),
            AggregateFunction::Median => "median".to_string(),
            AggregateFunction::Mean => "mean".to_string(),
        }
    }
}
