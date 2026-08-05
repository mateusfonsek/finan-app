-- Separates a checking account (bank statement) from a credit card (bill).
-- Drives how the account is shown (Import badge, listings) and enables future
-- queries like "card balance vs account balance".
ALTER TABLE accounts ADD COLUMN kind TEXT NOT NULL DEFAULT 'checking'
  CHECK(kind IN ('checking','credit_card'));
