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

/// Tudo relativo a categorias com kind='transfer' AND is_investment=1.
/// `saldo_acumulado` = soma de (aplicado - resgatado) ao longo de TODO o histórico
/// disponível na DB (não apenas o mês). Representa o capital líquido que entrou
/// no investimento, sem considerar rendimentos (que a OFX não traz).
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct InvestmentSummary {
    pub aplicado_no_mes: String,
    pub resgatado_no_mes: String,
    pub aplicacoes_count: u32,
    pub resgates_count: u32,
    pub saldo_acumulado: String,
}

/// Breakdown de transferências internas (kind='transfer' AND is_investment=0)
/// — pagamento de fatura, autotransferências, etc. Serve pra dar transparência
/// no Dashboard sobre o que foi excluído dos KPIs.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TransferSummary {
    pub total_out: String,
    pub total_in: String,
    pub count: u32,
}

/// Fonte de renda agregada (alguém que te paga). Substitui a ideia de
/// "categoria de renda" — pra entradas, o que importa é **quem pagou**.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct IncomeSource {
    /// Chave estável de agrupamento (CNPJ, CPF mascarado ou nome).
    pub key: String,
    /// Rótulo legível pra UI.
    pub label: String,
    /// Total recebido desta fonte no mês.
    pub total: String,
    /// Quantas tx desta fonte no mês.
    pub count: u32,
    /// % do total de entradas do mês.
    pub percent: f64,
    /// True se esta fonte apareceu em ≥2 meses distintos no histórico.
    pub is_recurring: bool,
    /// Quantos meses distintos esta fonte apareceu no histórico (all-time).
    pub recurring_months: u32,
}
