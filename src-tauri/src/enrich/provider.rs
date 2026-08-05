use crate::error::AppResult;

/// What any company lookup service must return. Deliberately minimal so a
/// non-Brazilian provider can satisfy it without inventing fields.
#[derive(Debug, Clone, Default)]
pub struct Company {
    pub legal_name: Option<String>,
    pub trade_name: Option<String>,
    /// Economic activity code (CNAE in Brazil). Matched against the locale
    /// pack's `rules.cnae_map` prefixes.
    pub activity_code: Option<String>,
    pub activity_label: Option<String>,
}

pub trait TaxIdProvider: Send + Sync {
    /// `tax_id_digits` is already stripped of punctuation. `Err` means "can't
    /// know right now", never "no such company" — callers treat both the same.
    fn lookup(&self, tax_id_digits: &str) -> AppResult<Company>;

    /// Milliseconds to wait between calls. A paid provider may return 0.
    fn courtesy_delay_ms(&self) -> u64 {
        250
    }
}

/// Resolves `manifest.taxId.provider` to an implementation. `None` for an
/// unknown or absent provider — that's what makes a locale without lookup a
/// no-op, with no country check anywhere else.
pub fn for_name(name: &str) -> Option<Box<dyn TaxIdProvider>> {
    match name {
        "brasilapi" => Some(Box::new(super::providers::brasilapi::BrasilApi)),
        _ => None,
    }
}
