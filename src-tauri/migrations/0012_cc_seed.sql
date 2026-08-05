-- Seed rules for Nubank credit-card OFX.
-- Idempotent: NOT EXISTS avoids duplicating a manual rule with the same pattern.
--
-- High priority (15) for received payments so they win; 8 for common merchants,
-- which loses to auto tax-id rules (10) and seed transfers (15) on conflict.

-- The bill payment as seen from the card side. Mirror of the checking-account
-- rule created in 0006. Both are kind=transfer.
INSERT INTO rules (pattern, category_id, priority, due_day, display_name)
SELECT 'Pagamento recebido', c.id, 15, NULL, 'Pagamento de fatura recebido (CC)'
FROM categories c
WHERE c.name = 'Transferências'
  AND NOT EXISTS (SELECT 1 FROM rules WHERE pattern = 'Pagamento recebido');

-- Common Brazilian card merchants. Case-insensitive LIKE substring matches
-- processor variations (Uber* Trip, Uber Uber *Trip Help.U, etc).

-- Transport
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

-- Restaurant / delivery
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

-- Subscription
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

-- Courses / SaaS / other
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
