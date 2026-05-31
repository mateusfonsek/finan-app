-- Distingue conta corrente (extrato bancário) de cartão de crédito (fatura).
-- Determina como o app exibe a conta (badge no Import, listagens) e dá base
-- pra queries futuras tipo "saldo do CC vs saldo da CA".
ALTER TABLE accounts ADD COLUMN kind TEXT NOT NULL DEFAULT 'checking'
  CHECK(kind IN ('checking','credit_card'));
