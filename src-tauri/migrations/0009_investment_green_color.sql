-- Investment reads as green (growth), not indigo. Applies the new
-- `--color-cat-investimento` token, an emerald distinct from market/income.
UPDATE categories
SET color_token = '--color-cat-investimento'
WHERE name = 'Investimentos';
