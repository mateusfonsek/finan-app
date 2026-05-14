use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub color_token: Option<String>,
    pub kind: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct NewCategory {
    pub name: String,
    pub color_token: Option<String>,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateCategory {
    pub name: String,
    pub color_token: Option<String>,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CategoryWithCount {
    pub id: i64,
    pub name: String,
    pub color_token: Option<String>,
    pub kind: String,
    pub created_at: String,
    pub transaction_count: u32,
}
