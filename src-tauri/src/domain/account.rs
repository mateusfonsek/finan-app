use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Account {
    pub id: i64,
    pub name: String,
    pub bank: Option<String>,
    pub ofx_acctid: Option<String>,
    /// `"checking"` or `"credit_card"`. Drives how the app displays and
    /// categorizes transactions from this account.
    pub kind: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct NewAccount {
    pub name: String,
    pub bank: Option<String>,
    pub ofx_acctid: Option<String>,
    /// Defaults to `"checking"` when the frontend omits it (back-compat).
    #[serde(default = "default_kind")]
    pub kind: String,
}

fn default_kind() -> String {
    "checking".to_string()
}
