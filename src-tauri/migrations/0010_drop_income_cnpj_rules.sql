-- Clears rules created for tax ids that only ever have INFLOW transactions.
-- Categorizing income makes no sense — it is tracked by counterparty in the
-- Dashboard's income sources panel, not by category.
--
-- Criterion: the pattern is a canonical CNPJ AND no outflow (amount<0) exists
-- whose description contains it. Idempotent.
DELETE FROM rules
WHERE pattern GLOB '[0-9][0-9].[0-9][0-9][0-9].[0-9][0-9][0-9]/[0-9][0-9][0-9][0-9]-[0-9][0-9]'
  AND NOT EXISTS (
    SELECT 1 FROM transactions t
    WHERE LOWER(t.description) LIKE '%' || LOWER(rules.pattern) || '%'
      AND CAST(t.amount AS REAL) < 0
  );
