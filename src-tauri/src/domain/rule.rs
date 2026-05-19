use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Rule {
    pub id: i64,
    pub pattern: String,
    pub category_id: i64,
    pub priority: i32,
    /// Dia do mês (1-31) em que a obrigação vence. NULL = sem prazo —
    /// a regra só aparece no calendário quando casa com uma transação.
    pub due_day: Option<i32>,
    /// Rótulo amigável da regra (ex: razão social vinda do CNPJ). NULL
    /// = nenhum rótulo definido; a UI cai pra `pattern`.
    pub display_name: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct NewRule {
    pub pattern: String,
    pub category_id: i64,
    pub priority: i32,
    pub due_day: Option<i32>,
    #[serde(default)]
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateRule {
    pub pattern: String,
    pub category_id: i64,
    pub priority: i32,
    pub due_day: Option<i32>,
    #[serde(default)]
    pub display_name: Option<String>,
}

/// Evento que aparece no calendário: combinação de uma regra
/// + (opcional) dia de vencimento + (opcional) transação que casou.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CalendarEvent {
    pub rule_id: i64,
    pub pattern: String,
    pub category_name: String,
    pub category_color_token: Option<String>,
    pub due_day: Option<i32>,
    pub paid_day: Option<i32>,
    pub paid_amount: Option<String>,
    pub paid_transaction_id: Option<i64>,
}
