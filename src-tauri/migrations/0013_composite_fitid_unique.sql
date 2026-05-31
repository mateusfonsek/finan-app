-- Migration 0013: chave única composta para transactions.
--
-- Motivação: Nubank reusa o mesmo FITID em transações semanticamente distintas
-- (compra original num mês, estorno dela no mês seguinte; IOF + valor principal
-- da mesma compra; parcelas N/12 e (N+1)/12 do mesmo plano). A UNIQUE original
-- `(account_id, ofx_fitid)` da migration 0001 trata colisões como duplicatas e
-- impede que ambas as pernas convivam — resultado: extrato fica meio importado,
-- KPIs distorcem (renda fantasma do estorno sem o gasto original, ou vice-versa).
--
-- Aqui relaxamos: duas linhas com mesmo FITID mas valor ou data diferentes são
-- transações distintas. SQLite não tem DROP CONSTRAINT, então recriamos a tabela.

CREATE TABLE transactions_new (
  id INTEGER PRIMARY KEY,
  account_id INTEGER NOT NULL REFERENCES accounts(id),
  date TEXT NOT NULL,
  amount TEXT NOT NULL,
  description TEXT NOT NULL,
  category_id INTEGER REFERENCES categories(id),
  notes TEXT,
  ofx_fitid TEXT,
  imported_at TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE(account_id, ofx_fitid, date, amount)
);

INSERT INTO transactions_new
  (id, account_id, date, amount, description, category_id, notes, ofx_fitid, imported_at)
SELECT
  id, account_id, date, amount, description, category_id, notes, ofx_fitid, imported_at
FROM transactions;

DROP TABLE transactions;
ALTER TABLE transactions_new RENAME TO transactions;

CREATE INDEX idx_tx_date ON transactions(date);
CREATE INDEX idx_tx_category ON transactions(category_id);
CREATE INDEX idx_tx_account ON transactions(account_id);
