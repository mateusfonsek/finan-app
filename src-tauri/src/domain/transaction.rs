use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Transaction {
    pub id: i64,
    pub account_id: i64,
    pub date: String,
    /// Decimal serialized as string (e.g. "-123.45"). Never f64.
    pub amount: String,
    pub description: String,
    pub category_id: Option<i64>,
    pub notes: Option<String>,
    pub ofx_fitid: Option<String>,
    pub imported_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct NewTransaction {
    pub date: String,
    /// Decimal as string. Backend converts via rust_decimal.
    pub amount: String,
    pub description: String,
    pub ofx_fitid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct InsertResult {
    pub inserted: u32,
    pub skipped_duplicates: u32,
}

impl NewTransaction {
    pub fn parse_amount(&self) -> Result<Decimal, rust_decimal::Error> {
        self.amount.parse::<Decimal>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn new_transaction_parses_positive_amount() {
        let t = NewTransaction {
            date: "2026-04-12".into(),
            amount: "123.45".into(),
            description: "salary".into(),
            ofx_fitid: Some("ABC123".into()),
        };
        assert_eq!(
            t.parse_amount().unwrap(),
            Decimal::from_str("123.45").unwrap()
        );
    }

    #[test]
    fn new_transaction_parses_negative_amount() {
        let t = NewTransaction {
            date: "2026-04-12".into(),
            amount: "-50.99".into(),
            description: "grocery".into(),
            ofx_fitid: Some("ABC124".into()),
        };
        assert_eq!(
            t.parse_amount().unwrap(),
            Decimal::from_str("-50.99").unwrap()
        );
    }

    #[test]
    fn new_transaction_rejects_garbage_amount() {
        let t = NewTransaction {
            date: "2026-04-12".into(),
            amount: "abc".into(),
            description: "x".into(),
            ofx_fitid: None,
        };
        assert!(t.parse_amount().is_err());
    }
}
