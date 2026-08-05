-- Composite unique key for transactions.
--
-- Nubank reuses the same FITID across semantically distinct transactions (a
-- purchase one month and its refund the next; IOF plus the principal of one
-- purchase; instalments N/12 and (N+1)/12 of one plan). The original
-- `(account_id, ofx_fitid)` UNIQUE from 0001 treats those collisions as
-- duplicates and blocks both legs — leaving a half-imported statement and
-- skewed KPIs (phantom income from a refund with no original expense).
--
-- Relaxed here: two rows with the same FITID but a different amount or date are
-- distinct transactions. SQLite has no DROP CONSTRAINT, so the table is rebuilt.

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
