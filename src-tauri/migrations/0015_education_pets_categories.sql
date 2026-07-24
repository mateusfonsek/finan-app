-- Duas categorias novas: Educação (mensalidade, cursos, autoescola, idiomas) e
-- Pets (veterinário, petshop). Ficam no banco de todo mundo (migration normal),
-- não só em instalações novas — assim usuários existentes também as recebem.
--
-- OR IGNORE: se o usuário já tiver criado manualmente uma categoria com esse
-- nome (ou key), a migration não sobrescreve.
INSERT OR IGNORE INTO categories (key, name, color_token, kind, is_investment) VALUES
  ('education', 'Educação', '--color-cat-indigo', 'expense', 0),
  ('pets',      'Pets',     '--color-cat-marrom', 'expense', 0);
