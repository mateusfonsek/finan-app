-- 'Transferências' category (kind=transfer): labels internal moves (card bill
-- payments, savings deposits/withdrawals) that are neither real spending nor
-- real income. KPIs and summaries exclude categories of this kind.
INSERT OR IGNORE INTO categories (name, color_token, kind)
VALUES ('Transferências', '--color-cat-outros', 'transfer');

-- Deterministic seed rules. NOT EXISTS avoids clashing with manual rules the
-- user may already have for the same patterns.
INSERT INTO rules (pattern, category_id, priority, due_day, display_name)
SELECT 'Pagamento de fatura', c.id, 15, NULL, 'Pagamento de fatura do cartão'
FROM categories c
WHERE c.name = 'Transferências'
  AND NOT EXISTS (SELECT 1 FROM rules WHERE pattern = 'Pagamento de fatura');

INSERT INTO rules (pattern, category_id, priority, due_day, display_name)
SELECT 'Aplicação RDB', c.id, 15, NULL, 'Aplicação em poupança/RDB'
FROM categories c
WHERE c.name = 'Transferências'
  AND NOT EXISTS (SELECT 1 FROM rules WHERE pattern = 'Aplicação RDB');

INSERT INTO rules (pattern, category_id, priority, due_day, display_name)
SELECT 'Resgate RDB', c.id, 15, NULL, 'Resgate de poupança/RDB'
FROM categories c
WHERE c.name = 'Transferências'
  AND NOT EXISTS (SELECT 1 FROM rules WHERE pattern = 'Resgate RDB');
