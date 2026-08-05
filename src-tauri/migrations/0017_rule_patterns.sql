-- Uma regra passa a ter N descrições em OR: "se a descrição contém A OU B,
-- use a categoria Y". Bancos descrevem o mesmo débito de formas diferentes
-- (débito automático vs. pagamento de boleto), e antes isso obrigava a
-- duplicar a regra inteira só pra variar o trecho procurado.
CREATE TABLE rule_patterns (
  id      INTEGER PRIMARY KEY,
  rule_id INTEGER NOT NULL REFERENCES rules(id) ON DELETE CASCADE,
  pattern TEXT NOT NULL
);

-- Cada regra existente vira uma regra de uma descrição só.
INSERT INTO rule_patterns (rule_id, pattern) SELECT id, pattern FROM rules;

CREATE INDEX idx_rule_patterns_rule ON rule_patterns(rule_id);

-- Fonte de verdade única: manter a coluna antiga espelhada é como o
-- display_name acabou sendo apagado silenciosamente em toda edição.
ALTER TABLE rules DROP COLUMN pattern;
