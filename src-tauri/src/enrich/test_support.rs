//! Apoio de teste para o enriquecimento. Compilado apenas em `cfg(test)`.

use std::collections::HashMap;
use std::sync::Mutex;

use crate::enrich::provider::{Company, TaxIdProvider};
use crate::error::{AppError, AppResult};

/// Provedor determinístico: responde a partir de um mapa em memória e registra
/// cada chamada, para os testes afirmarem *quantas* consultas houve — que é
/// metade do que o cancelamento e o pulo por regra existente precisam provar.
pub struct FakeProvider {
    /// dígitos do CNPJ → código de atividade devolvido.
    pub by_digits: HashMap<String, String>,
    pub calls: Mutex<Vec<String>>,
}

impl FakeProvider {
    pub fn new(pairs: &[(&str, &str)]) -> Self {
        Self {
            by_digits: pairs
                .iter()
                .map(|(d, c)| ((*d).to_string(), (*c).to_string()))
                .collect(),
            calls: Mutex::new(Vec::new()),
        }
    }

    pub fn call_count(&self) -> usize {
        self.calls.lock().expect("calls mutex poisoned").len()
    }
}

impl TaxIdProvider for FakeProvider {
    fn lookup(&self, tax_id_digits: &str) -> AppResult<Company> {
        self.calls
            .lock()
            .expect("calls mutex poisoned")
            .push(tax_id_digits.to_string());
        match self.by_digits.get(tax_id_digits) {
            Some(code) => Ok(Company {
                legal_name: Some(format!("EMPRESA {tax_id_digits}")),
                trade_name: None,
                activity_code: Some(code.clone()),
                activity_label: None,
            }),
            // Ausente = "não dá pra saber agora", que é o contrato do trait.
            None => Err(AppError::Invalid(format!(
                "fake: sem resposta para {tax_id_digits}"
            ))),
        }
    }

    /// Zero: nenhum teste deve pagar o pedágio de cortesia.
    fn courtesy_delay_ms(&self) -> u64 {
        0
    }
}
