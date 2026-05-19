-- Remove a categoria 'Renda' (vestigial).
-- Renda passou a ser rastreada por contraparte ("Fontes de Renda" no Dashboard)
-- em vez de categoria. A categoria não tem mais propósito funcional — nenhum
-- código checa `kind='income'`, era decorativa.
--
-- Idempotente: se Renda já foi removida, todos os comandos viram no-op.

-- Tira o link de tx que estavam categorizadas como Renda (mantém as tx).
UPDATE transactions
SET category_id = NULL
WHERE category_id IN (SELECT id FROM categories WHERE name = 'Renda');

-- Apaga regras que apontavam pra Renda (raro, mas seguro).
DELETE FROM rules
WHERE category_id IN (SELECT id FROM categories WHERE name = 'Renda');

-- Remove a categoria.
DELETE FROM categories WHERE name = 'Renda';
