use serde::{Deserialize, Serialize};
use specta::Type;

/// Nomes das categorias criadas pelo migration 0001. Não podem ser deletadas
/// pra preservar referências e expectativas do usuário. Podem ser renomeadas
/// (o que efetivamente as remove desta lista até o nome bater de novo).
pub const DEFAULT_CATEGORY_NAMES: &[&str] = &[
    "Mercado",
    "Restaurante",
    "Transporte",
    "Casa",
    "Saúde",
    "Lazer",
    "Assinatura",
    "Renda",
    "Outros",
];

pub fn is_default_category(name: &str) -> bool {
    DEFAULT_CATEGORY_NAMES.iter().any(|n| *n == name)
}

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
    pub is_default: bool,
}
