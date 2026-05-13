use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Account {
    pub id: i64,
    pub name: String,
    pub bank: Option<String>,
    pub ofx_acctid: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct NewAccount {
    pub name: String,
    pub bank: Option<String>,
    pub ofx_acctid: Option<String>,
}
