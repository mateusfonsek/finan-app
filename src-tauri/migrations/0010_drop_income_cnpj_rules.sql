-- Limpa regras criadas pra CNPJs que só têm tx de ENTRADA (renda).
-- Categorizar renda não faz sentido — renda é rastreada por contraparte
-- na seção "Fontes de Renda" do Dashboard, não por categoria.
--
-- Critério: pattern é um CNPJ no formato canônico AND não existe tx de saída
-- (amount<0) cuja description contém esse CNPJ. Idempotente.
DELETE FROM rules
WHERE pattern GLOB '[0-9][0-9].[0-9][0-9][0-9].[0-9][0-9][0-9]/[0-9][0-9][0-9][0-9]-[0-9][0-9]'
  AND NOT EXISTS (
    SELECT 1 FROM transactions t
    WHERE LOWER(t.description) LIKE '%' || LOWER(rules.pattern) || '%'
      AND CAST(t.amount AS REAL) < 0
  );
