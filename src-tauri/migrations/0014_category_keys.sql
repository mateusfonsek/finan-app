-- Chave estável, independente de idioma, pra cada categoria. É o pivô entre a
-- lógica de classificação (cnae_map, seed_rules do locale pack) e o nome exibido:
-- as regras referenciam a `key`, nunca o `name`, então classificar continua
-- funcionando em qualquer idioma.
--
-- Categorias criadas pelo usuário ficam com key NULL (o índice único permite
-- múltiplos NULLs no SQLite) — elas não participam da classificação automática.
ALTER TABLE categories ADD COLUMN key TEXT;

-- Backfill das categorias seedadas pelas migrations, pelo nome PT atual.
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
