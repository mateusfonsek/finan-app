use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Rule {
    pub id: i64,
    /// Trechos procurados na descrição, em OR: a regra casa quando a descrição
    /// contém QUALQUER um deles. Sempre com pelo menos um item.
    pub patterns: Vec<String>,
    pub category_id: i64,
    pub priority: i32,
    /// Dia do mês (1-31) em que a obrigação vence. NULL = sem prazo —
    /// a regra só aparece no calendário quando casa com uma transação.
    pub due_day: Option<i32>,
    /// Rótulo amigável da regra (ex: razão social vinda do CNPJ). NULL
    /// = nenhum rótulo definido; a UI cai pro primeiro pattern.
    pub display_name: Option<String>,
    pub created_at: String,
}

/// Uma regra + quantas transações ela alcança. Serve à tela de Regras, onde a
/// pergunta é "essa regra está pegando alguma coisa?".
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RuleWithCount {
    pub id: i64,
    pub patterns: Vec<String>,
    pub category_id: i64,
    pub priority: i32,
    pub due_day: Option<i32>,
    pub display_name: Option<String>,
    pub created_at: String,
    /// Transações cuja descrição casa QUALQUER trecho da regra, independente
    /// da categoria em que estão hoje. É alcance, não autoria: uma transação
    /// que você categorizou na mão continua contando, e uma que outra regra de
    /// prioridade maior levou também.
    pub transaction_count: u32,
}

/// Uma mudança que aplicar as regras causaria numa transação. É o material da
/// tela de revisão: nada é gravado até o usuário escolher.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RulePreviewRow {
    pub transaction_id: i64,
    pub date: String,
    /// Decimal como string, igual ao resto do app. Nunca f64.
    pub amount: String,
    pub description: String,
    /// `None` = a transação está sem categoria. Qualquer outra coisa significa
    /// que aplicar a regra SUBSTITUI a categoria atual.
    ///
    /// Atenção: o banco não registra quem definiu a categoria, então isto não
    /// distingue "você escolheu na mão" de "uma regra anterior definiu". A UI
    /// não pode afirmar autoria — só que existe categoria e ela mudaria.
    pub current_category_id: Option<i64>,
    pub new_category_id: i64,
    pub rule_id: i64,
    /// Rótulo da regra que venceu: `display_name` quando existe, senão o trecho
    /// que de fato casou esta descrição.
    pub rule_label: String,
}

/// Escolha do usuário na tela de revisão: esta transação vai pra esta categoria.
///
/// A categoria vem explícita em vez de recalculada no momento de aplicar — se
/// as regras mudarem entre a revisão e o clique, grava-se o que foi revisado, e
/// não uma surpresa.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RuleChoice {
    pub transaction_id: i64,
    pub category_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct NewRule {
    pub patterns: Vec<String>,
    pub category_id: i64,
    pub priority: i32,
    pub due_day: Option<i32>,
    #[serde(default)]
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UpdateRule {
    pub patterns: Vec<String>,
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
    /// O trecho que de fato casou a transação — ou o primeiro da regra, quando
    /// o evento existe só pelo `due_day` e nada casou ainda.
    pub pattern: String,
    pub category_name: String,
    pub category_color_token: Option<String>,
    pub due_day: Option<i32>,
    pub paid_day: Option<i32>,
    pub paid_amount: Option<String>,
    pub paid_transaction_id: Option<i64>,
}
