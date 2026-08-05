-- Automatic import: the user picks folders, the app scans for new .ofx files
-- and notifies. Nothing is imported without the preview — these tables hold
-- only the configuration and what has already been seen.

-- Generic key/value store for app preferences. Born here for `watch_enabled`,
-- but usable for any future setting.
CREATE TABLE app_settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

-- Watched folders. `path` is canonicalized before INSERT, so UNIQUE rejects
-- the same folder added twice via different paths (symlink, /tmp vs
-- /private/tmp).
CREATE TABLE watched_folders (
  id INTEGER PRIMARY KEY,
  path TEXT NOT NULL UNIQUE,
  label TEXT NOT NULL,
  added_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Files already seen, keyed by CONTENT HASH rather than path. That handles for
-- free: the same statement downloaded twice under different names, a renamed
-- file, and a file moved between two watched folders.
--
-- status:
--   pending  - discovered, feeds the badge, awaiting the user's decision
--   imported - the user imported it
--   ignored  - the user dismissed it (permanent, never notified again)
--   invalid  - does not parse as OFX (silently never notified again)
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
