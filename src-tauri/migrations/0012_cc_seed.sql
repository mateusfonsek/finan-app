-- Seed rules pra OFX de cartão de crédito Nubank.
-- Idempotente: NOT EXISTS evita duplicar quando o usuário já criou regra
-- manual com o mesmo pattern.
--
-- Priority alta (15) pra Pagamento recebido (vence outras), 8 pros merchants
-- comuns (perde pra CNPJ-auto=10 e seed-transfer=15 caso conflitem).

-- Pagamento da fatura visto pelo lado do cartão. Equivalente do
-- "Pagamento de fatura" da CA (já criado em 0006). Ambos kind=transfer.
INSERT INTO rules (pattern, category_id, priority, due_day, display_name)
SELECT 'Pagamento recebido', c.id, 15, NULL, 'Pagamento de fatura recebido (CC)'
FROM categories c
WHERE c.name = 'Transferências'
  AND NOT EXISTS (SELECT 1 FROM rules WHERE pattern = 'Pagamento recebido');

-- Merchants comuns no cartão BR. LIKE substring case-insensitive casa
-- variações do processador (Uber* Trip, Uber Uber *Trip Help.U, etc).

-- Transporte
INSERT INTO rules (pattern, category_id, priority, due_day, display_name)
SELECT 'UBER', c.id, 8, NULL, 'Uber'
FROM categories c
WHERE c.name = 'Transporte'
  AND NOT EXISTS (SELECT 1 FROM rules WHERE pattern = 'UBER');

INSERT INTO rules (pattern, category_id, priority, due_day, display_name)
SELECT '99APP', c.id, 8, NULL, '99 App'
FROM categories c
WHERE c.name = 'Transporte'
  AND NOT EXISTS (SELECT 1 FROM rules WHERE pattern = '99APP');

INSERT INTO rules (pattern, category_id, priority, due_day, display_name)
SELECT 'Dl*Uberrides', c.id, 8, NULL, 'Uber (Dl*)'
FROM categories c
WHERE c.name = 'Transporte'
  AND NOT EXISTS (SELECT 1 FROM rules WHERE pattern = 'Dl*Uberrides');

-- Restaurante / delivery
INSERT INTO rules (pattern, category_id, priority, due_day, display_name)
SELECT 'IFOOD', c.id, 8, NULL, 'iFood'
FROM categories c
WHERE c.name = 'Restaurante'
  AND NOT EXISTS (SELECT 1 FROM rules WHERE pattern = 'IFOOD');

INSERT INTO rules (pattern, category_id, priority, due_day, display_name)
SELECT 'Ifd*', c.id, 8, NULL, 'iFood (Ifd*)'
FROM categories c
WHERE c.name = 'Restaurante'
  AND NOT EXISTS (SELECT 1 FROM rules WHERE pattern = 'Ifd*');

-- Assinatura
INSERT INTO rules (pattern, category_id, priority, due_day, display_name)
SELECT 'NETFLIX', c.id, 8, NULL, 'Netflix'
FROM categories c
WHERE c.name = 'Assinatura'
  AND NOT EXISTS (SELECT 1 FROM rules WHERE pattern = 'NETFLIX');

INSERT INTO rules (pattern, category_id, priority, due_day, display_name)
SELECT 'SPOTIFY', c.id, 8, NULL, 'Spotify'
FROM categories c
WHERE c.name = 'Assinatura'
  AND NOT EXISTS (SELECT 1 FROM rules WHERE pattern = 'SPOTIFY');

INSERT INTO rules (pattern, category_id, priority, due_day, display_name)
SELECT 'HBO', c.id, 8, NULL, 'HBO Max'
FROM categories c
WHERE c.name = 'Assinatura'
  AND NOT EXISTS (SELECT 1 FROM rules WHERE pattern = 'HBO');

INSERT INTO rules (pattern, category_id, priority, due_day, display_name)
SELECT 'AMAZON PRIME', c.id, 8, NULL, 'Amazon Prime'
FROM categories c
WHERE c.name = 'Assinatura'
  AND NOT EXISTS (SELECT 1 FROM rules WHERE pattern = 'AMAZON PRIME');

-- Cursos / SaaS / outros
INSERT INTO rules (pattern, category_id, priority, due_day, display_name)
SELECT 'HUBLA', c.id, 8, NULL, 'Hubla (curso)'
FROM categories c
WHERE c.name = 'Outros'
  AND NOT EXISTS (SELECT 1 FROM rules WHERE pattern = 'HUBLA');

INSERT INTO rules (pattern, category_id, priority, due_day, display_name)
SELECT 'UDEMY', c.id, 8, NULL, 'Udemy (curso)'
FROM categories c
WHERE c.name = 'Outros'
  AND NOT EXISTS (SELECT 1 FROM rules WHERE pattern = 'UDEMY');
