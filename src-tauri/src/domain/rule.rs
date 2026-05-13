use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Rule {
    pub id: i64,
    pub pattern: String,
    pub category_id: i64,
    pub priority: i32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct NewRule {
    pub pattern: String,
    pub category_id: i64,
    pub priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateRule {
    pub pattern: String,
    pub category_id: i64,
    pub priority: i32,
}
