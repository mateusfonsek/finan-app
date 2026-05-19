-- Investimento é semanticamente "verde" (crescimento patrimonial), não índigo.
-- Aplica o token novo `--color-cat-investimento` (emerald distinto do mercado/renda).
UPDATE categories
SET color_token = '--color-cat-investimento'
WHERE name = 'Investimentos';
