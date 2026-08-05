-- A rule now holds N snippets OR'd together: "if the description contains A OR
-- B, use category Y". Banks describe the same charge in different ways (direct
-- debit vs bill payment), which previously forced duplicating the whole rule
-- just to vary the searched snippet.
CREATE TABLE rule_patterns (
  id      INTEGER PRIMARY KEY,
  rule_id INTEGER NOT NULL REFERENCES rules(id) ON DELETE CASCADE,
  pattern TEXT NOT NULL
);

-- Every existing rule becomes a single-snippet rule.
INSERT INTO rule_patterns (rule_id, pattern) SELECT id, pattern FROM rules;

CREATE INDEX idx_rule_patterns_rule ON rule_patterns(rule_id);

-- Single source of truth: keeping the old column mirrored is how display_name
-- ended up silently wiped on every edit.
ALTER TABLE rules DROP COLUMN pattern;
