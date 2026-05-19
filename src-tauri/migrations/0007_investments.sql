-- Flag pra distinguir categorias de investimento das outras transferências.
-- kind='transfer' + is_investment=1 → Investimentos (visualizadas em seção própria)
-- kind='transfer' + is_investment=0 → Transferências internas (pagamento de fatura etc)
-- Ambas saem dos KPIs de Renda/Gastos, mas só Investimentos têm acompanhamento dedicado.
ALTER TABLE categories ADD COLUMN is_investment INTEGER NOT NULL DEFAULT 0;

-- Categoria Investimentos (cor indigo distinta).
INSERT OR IGNORE INTO categories (name, color_token, kind, is_investment)
VALUES ('Investimentos', '--color-cat-indigo', 'transfer', 1);

-- Reaponta as regras seed do RDB (criadas em 0006) pra Investimentos.
UPDATE rules
SET category_id = (SELECT id FROM categories WHERE name = 'Investimentos'),
    display_name = CASE pattern
      WHEN 'Aplicação RDB' THEN 'Aplicação em investimento'
      WHEN 'Resgate RDB' THEN 'Resgate de investimento'
      ELSE display_name
    END
WHERE pattern IN ('Aplicação RDB', 'Resgate RDB')
  AND EXISTS (SELECT 1 FROM categories WHERE name = 'Investimentos');

-- Reclassifica tx existentes que estavam em Transferências por causa do RDB.
UPDATE transactions
SET category_id = (SELECT id FROM categories WHERE name = 'Investimentos')
WHERE id IN (
  SELECT t.id FROM transactions t
  JOIN categories c ON c.id = t.category_id
  WHERE c.name = 'Transferências'
    AND (t.description LIKE 'Aplicação RDB%' OR t.description LIKE 'Resgate RDB%')
);
