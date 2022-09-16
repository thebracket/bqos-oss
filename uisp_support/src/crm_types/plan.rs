use serde::Deserialize;

#[allow(non_snake_case)]
#[derive(Deserialize, Debug, Clone)]
pub struct ServicePlan {
    pub name: Option<String>,
    pub organizationId: Option<usize>,
    pub invoiceLabel: Option<String>,
    pub downloadBurst: Option<f32>,
    pub uploadBurst: Option<f32>,
    pub downloadSpeed: Option<f32>,
    pub uploadSpeed: Option<f32>,
    pub dataUsageLimit: Option<f32>,
    pub aggregation: Option<usize>,
    pub taxable: Option<bool>,
    pub amountExemptFromTaxation: Option<f32>,
    pub setupFee: Option<f32>,
    pub earlyTerminationFee: Option<f32>,
    pub minimumContractLengthMonths: Option<usize>,
    pub public: Option<bool>,
    pub id: usize,
    pub periods: Vec<PlanPeriod>,
    pub servicePlanType: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug, Clone)]
pub struct PlanPeriod {
    pub period: Option<usize>,
    pub price: Option<f32>,
    pub enabled: Option<bool>,
    pub id: Option<usize>,
}
