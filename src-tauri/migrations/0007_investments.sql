-- Flag separating investment categories from other transfers.
-- kind='transfer' + is_investment=1 -> Investments (own dashboard section)
-- kind='transfer' + is_investment=0 -> internal transfers (card bill, etc)
-- Both leave the income/spending KPIs; only investments get dedicated tracking.
ALTER TABLE categories ADD COLUMN is_investment INTEGER NOT NULL DEFAULT 0;

-- Investments category (distinct indigo colour).
INSERT OR IGNORE INTO categories (name, color_token, kind, is_investment)
VALUES ('Investimentos', '--color-cat-indigo', 'transfer', 1);

-- Repoints the RDB seed rules from 0006 to Investments.
UPDATE rules
SET category_id = (SELECT id FROM categories WHERE name = 'Investimentos'),
    display_name = CASE pattern
      WHEN 'Aplicação RDB' THEN 'Aplicação em investimento'
      WHEN 'Resgate RDB' THEN 'Resgate de investimento'
      ELSE display_name
    END
WHERE pattern IN ('Aplicação RDB', 'Resgate RDB')
  AND EXISTS (SELECT 1 FROM categories WHERE name = 'Investimentos');

-- Reclassifies existing tx that landed in Transfers because of RDB.
UPDATE transactions
SET category_id = (SELECT id FROM categories WHERE name = 'Investimentos')
WHERE id IN (
  SELECT t.id FROM transactions t
  JOIN categories c ON c.id = t.category_id
  WHERE c.name = 'Transferências'
    AND (t.description LIKE 'Aplicação RDB%' OR t.description LIKE 'Resgate RDB%')
);
