-- Removes the vestigial 'Renda' category.
-- Income is now tracked by counterparty (the Dashboard's income sources panel)
-- rather than by category. No code checks `kind='income'` — it was decorative.
--
-- Idempotent: if it is already gone, every statement is a no-op.

-- Unlinks tx categorized as Renda, keeping the tx.
UPDATE transactions
SET category_id = NULL
WHERE category_id IN (SELECT id FROM categories WHERE name = 'Renda');

-- Drops rules pointing at Renda (rare, but safe).
DELETE FROM rules
WHERE category_id IN (SELECT id FROM categories WHERE name = 'Renda');

-- Removes the category.
DELETE FROM categories WHERE name = 'Renda';
