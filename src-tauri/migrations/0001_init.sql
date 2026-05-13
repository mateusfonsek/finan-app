CREATE TABLE accounts (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  bank TEXT,
  ofx_acctid TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE categories (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  color_token TEXT,
  kind TEXT NOT NULL CHECK(kind IN ('expense','income','transfer')),
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE transactions (
  id INTEGER PRIMARY KEY,
  account_id INTEGER NOT NULL REFERENCES accounts(id),
  date TEXT NOT NULL,
  amount TEXT NOT NULL,
  description TEXT NOT NULL,
  category_id INTEGER REFERENCES categories(id),
  notes TEXT,
  ofx_fitid TEXT,
  imported_at TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE(account_id, ofx_fitid)
);

CREATE INDEX idx_tx_date ON transactions(date);
CREATE INDEX idx_tx_category ON transactions(category_id);
CREATE INDEX idx_tx_account ON transactions(account_id);

INSERT INTO categories (name, color_token, kind) VALUES
  ('Mercado',     '--color-cat-mercado',     'expense'),
  ('Restaurante', '--color-cat-restaurante', 'expense'),
  ('Transporte',  '--color-cat-transporte',  'expense'),
  ('Casa',        '--color-cat-casa',        'expense'),
  ('Saúde',       '--color-cat-saude',       'expense'),
  ('Lazer',       '--color-cat-lazer',       'expense'),
  ('Assinatura',  '--color-cat-assinatura',  'expense'),
  ('Renda',       '--color-cat-renda',       'income'),
  ('Outros',      '--color-cat-outros',      'expense');
