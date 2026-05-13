use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct KpiSummary {
    pub income: String,
    pub expense: String,
    pub net: String,
    pub transaction_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CategorySpend {
    pub category_id: Option<i64>,
    pub name: String,
    pub color_token: Option<String>,
    pub total: String,
    pub percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MonthSummary {
    pub month: String,
    pub income: String,
    pub expense: String,
}
