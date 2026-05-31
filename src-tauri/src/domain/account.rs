use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Account {
    pub id: i64,
    pub name: String,
    pub bank: Option<String>,
    pub ofx_acctid: Option<String>,
    /// `"checking"` (conta corrente) ou `"credit_card"` (cartão de crédito).
    /// Distingue como o app exibe e categoriza tx originárias dessa conta.
    pub kind: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct NewAccount {
    pub name: String,
    pub bank: Option<String>,
    pub ofx_acctid: Option<String>,
    /// Default `"checking"` quando o frontend não enviar (compat retroativa).
    #[serde(default = "default_kind")]
    pub kind: String,
}

fn default_kind() -> String {
    "checking".to_string()
}
