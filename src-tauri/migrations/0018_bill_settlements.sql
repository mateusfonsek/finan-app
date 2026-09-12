-- Which month this bill's payment lands in, relative to its due date. 0 = paid
-- in the month it falls due; 1 = paid the month before, which is what happens
-- when the statement arrives before the cycle closes.
--
-- Whole months, not a window of days: in a monthly cycle a day window cannot
-- tell "paid early" from "paid late for the previous occurrence" — a payment on
-- the 15th sits 22 days before one due date and 9 days after the other.
ALTER TABLE rules ADD COLUMN pay_lead_months INTEGER NOT NULL DEFAULT 0;

-- The user saying what the statement has no way to say.
--
-- The row IS the state: its existence means "this occurrence is settled". There
-- is no `status` column and no way to mark one unpaid — that would contradict
-- the statement, which owns the amounts.
--
-- A NULL transaction_id means paid outside the statement (cash, or someone else
-- paid). No amount is stored: money that never passed through the account
-- belongs in no KPI.
CREATE TABLE bill_settlements (
  id             INTEGER PRIMARY KEY,
  rule_id        INTEGER NOT NULL REFERENCES rules(id) ON DELETE CASCADE,
  due_month      TEXT NOT NULL,
  transaction_id INTEGER REFERENCES transactions(id) ON DELETE SET NULL,
  settled_at     TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE(rule_id, due_month)
);
