pub enum QueryFilter {
    MatchOne {
        field: String,
        value: String,
    },
    Either {
        field: String,
        value_1: String,
        value_2: String,
    },
}

impl QueryFilter {
    pub fn to_query_string(&self) -> String {
        match self {
            QueryFilter::MatchOne{field, value} => format!("|> filter(fn: (r) => r[\"{field}\"] == \"{value}\")\n"),
            QueryFilter::Either{field, value_1, value_2} => format!("|> filter(fn: (r) => r[\"{field}\"] == \"{value_1}\" or r[\"{field}\"] == \"{value_2}\")\n"),
        }
    }
}
