-- Categoria 'Transferências' (kind=transfer) — usada pra rotular movimentações
-- internas (pagamento de fatura, aplicação/resgate em poupança) que NÃO são
-- gasto nem renda real. KPIs e summaries excluem categorias desse kind.
INSERT OR IGNORE INTO categories (name, color_token, kind)
VALUES ('Transferências', '--color-cat-outros', 'transfer');

-- Seed de regras determinísticas. NOT EXISTS evita conflito com regras manuais
-- que o usuário possa ter criado pros mesmos patterns.
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
