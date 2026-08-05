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

/// Everything about categories with kind='transfer' AND is_investment=1.
/// `saldo_acumulado` sums (deposited - withdrawn) over the WHOLE history in the
/// DB, not just the month. It is net capital put in, excluding returns, which
/// OFX does not carry.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct InvestmentSummary {
    pub aplicado_no_mes: String,
    pub resgatado_no_mes: String,
    pub aplicacoes_count: u32,
    pub resgates_count: u32,
    pub saldo_acumulado: String,
}

/// Breakdown of internal transfers (kind='transfer' AND is_investment=0)
/// — card bill payments, self-transfers, etc. Gives the Dashboard a way to be
/// transparent about what was excluded from the KPIs.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TransferSummary {
    pub total_out: String,
    pub total_in: String,
    pub count: u32,
}

/// An aggregated income source (someone who pays you). Replaces the idea of an
/// "income category" — for inflows what matters is **who paid**.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct IncomeSource {
    /// Stable grouping key (tax id, masked personal id, or name).
    pub key: String,
    /// Readable label for the UI.
    pub label: String,
    /// Total received from this source in the month.
    pub total: String,
    /// How many tx from this source in the month.
    pub count: u32,
    /// Share of the month's total inflows.
    pub percent: f64,
    /// True when this source appeared in >= 2 distinct months of history.
    pub is_recurring: bool,
    /// How many distinct months this source appeared in, all-time.
    pub recurring_months: u32,
}
