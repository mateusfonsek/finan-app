-- Importação automática: o usuário escolhe pastas, o app varre à procura de
-- .ofx novos e avisa. Nada é importado sem o preview — estas tabelas guardam
-- só a configuração e o que já foi visto.

-- Chave/valor genérico pra preferências do app. Nasce aqui por causa de
-- `watch_enabled`, mas serve pra qualquer pref futura.
CREATE TABLE app_settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

-- Pastas observadas. `path` é canonicalizado antes do INSERT, então o UNIQUE
-- barra a mesma pasta adicionada duas vezes por caminhos diferentes
-- (symlink, /tmp vs /private/tmp).
CREATE TABLE watched_folders (
  id INTEGER PRIMARY KEY,
  path TEXT NOT NULL UNIQUE,
  label TEXT NOT NULL,
  added_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Arquivos já vistos, chaveados pelo HASH DO CONTEÚDO e não pelo caminho.
-- É isso que resolve de graça: mesmo extrato baixado 2x com nomes diferentes,
-- arquivo renomeado, e arquivo movido entre duas pastas observadas.
--
-- status:
--   pending  — descoberto, alimenta o badge, aguarda decisão do usuário
--   imported — o usuário importou
--   ignored  — o usuário dispensou (definitivo, nunca mais avisa)
--   invalid  — não parseia como OFX (nunca mais avisa, silenciosamente)
CREATE TABLE seen_files (
  id INTEGER PRIMARY KEY,
  content_hash TEXT NOT NULL UNIQUE,
  path TEXT NOT NULL,
  file_name TEXT NOT NULL,
  size INTEGER NOT NULL,
  status TEXT NOT NULL CHECK(status IN ('pending','imported','ignored','invalid')),
  seen_at TEXT NOT NULL DEFAULT (datetime('now')),
  resolved_at TEXT
);

CREATE INDEX idx_seen_files_status ON seen_files(status);
