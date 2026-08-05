-- A stable, language-independent key per category. It is the pivot between
-- classification logic (the locale pack's cnae_map and seed_rules) and the
-- displayed name: rules reference `key`, never `name`, so classification keeps
-- working in any language.
--
-- User-created categories keep a NULL key (SQLite's unique index allows many
-- NULLs) and take no part in automatic classification.
ALTER TABLE categories ADD COLUMN key TEXT;

-- Backfills migration-seeded categories by their current Portuguese name.
UPDATE categories SET key = 'market'       WHERE key IS NULL AND name = 'Mercado';
UPDATE categories SET key = 'restaurant'   WHERE key IS NULL AND name = 'Restaurante';
UPDATE categories SET key = 'transport'    WHERE key IS NULL AND name = 'Transporte';
UPDATE categories SET key = 'home'         WHERE key IS NULL AND name = 'Casa';
UPDATE categories SET key = 'health'       WHERE key IS NULL AND name = 'Saúde';
UPDATE categories SET key = 'leisure'      WHERE key IS NULL AND name = 'Lazer';
UPDATE categories SET key = 'subscription' WHERE key IS NULL AND name = 'Assinatura';
UPDATE categories SET key = 'shopping'     WHERE key IS NULL AND name = 'Compras';
UPDATE categories SET key = 'other'        WHERE key IS NULL AND name = 'Outros';
UPDATE categories SET key = 'transfer'     WHERE key IS NULL AND name = 'Transferências';
UPDATE categories SET key = 'investment'   WHERE key IS NULL AND name = 'Investimentos';

CREATE UNIQUE INDEX idx_categories_key ON categories(key);
