-- Two new categories: Education (tuition, courses, driving school, languages)
-- and Pets (vet, pet shop). A normal migration so existing installs get them
-- too, not just fresh ones.
--
-- OR IGNORE: does not overwrite a category the user already created with that
-- name or key.
INSERT OR IGNORE INTO categories (key, name, color_token, kind, is_investment) VALUES
  ('education', 'Educação', '--color-cat-indigo', 'expense', 0),
  ('pets',      'Pets',     '--color-cat-marrom', 'expense', 0);
