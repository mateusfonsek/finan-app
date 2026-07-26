//! Importação automática — descoberta de extratos `.ofx` em pastas escolhidas
//! pelo usuário.
//!
//! Não existe watcher de filesystem aqui, de propósito: o usuário manda o
//! arquivo do celular *e então vai olhar o Mac*, então o foco da janela é o
//! gatilho natural. Isso evita a dependência `notify`, evita interpretar
//! eventos de FS e evita o problema conhecido de FSEvents não disparar de
//! forma confiável dentro do iCloud Drive.

use std::path::Path;

use rusqlite::{params, Connection};
use serde::Serialize;
use specta::Type;
use tauri::State;

use crate::db::Db;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Type)]
pub struct WatchedFolder {
    pub id: i64,
    pub path: String,
    /// Nome curto pra exibir ("finan", "Downloads"). Derivado do último
    /// componente do caminho no momento em que a pasta foi adicionada.
    pub label: String,
    /// `false` quando a pasta sumiu ou foi desmontada — a linha vira estado de
    /// erro na UI, mas as outras pastas seguem funcionando.
    pub exists: bool,
    pub imported_count: i64,
    pub last_imported_at: Option<String>,
}

/// Nome exibido a partir do caminho. Cai em "/" só em caminho degenerado.
fn label_for(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string())
}

/// Resolve symlinks e normaliza (`/tmp` → `/private/tmp` no macOS) pra que o
/// UNIQUE da tabela realmente barre a mesma pasta adicionada duas vezes.
fn canonical(path: &str) -> AppResult<String> {
    let p = std::fs::canonicalize(path).map_err(|e| {
        AppError::Path(format!("não consegui resolver o caminho '{path}': {e}"))
    })?;
    if !p.is_dir() {
        return Err(AppError::Invalid(format!("'{path}' não é uma pasta")));
    }
    Ok(p.to_string_lossy().to_string())
}

fn row_to_folder(row: &rusqlite::Row) -> rusqlite::Result<WatchedFolder> {
    let path: String = row.get(1)?;
    let exists = Path::new(&path).is_dir();
    Ok(WatchedFolder {
        id: row.get(0)?,
        path,
        label: row.get(2)?,
        exists,
        imported_count: row.get(3)?,
        last_imported_at: row.get(4)?,
    })
}

/// A contagem de importados por pasta usa `LIKE '<path>/%'` sobre o último
/// caminho conhecido de cada arquivo — é evidência pro usuário ("3 extratos
/// importados"), não contabilidade: um arquivo movido depois do import migra
/// de pasta na contagem, e tudo bem.
const SELECT_FOLDERS: &str = "
    SELECT w.id,
           w.path,
           w.label,
           (SELECT COUNT(*) FROM seen_files s
             WHERE s.status = 'imported' AND s.path LIKE w.path || '/%'),
           (SELECT MAX(s.resolved_at) FROM seen_files s
             WHERE s.status = 'imported' AND s.path LIKE w.path || '/%')
      FROM watched_folders w
     ORDER BY w.added_at";

pub fn list_folders(conn: &Connection) -> AppResult<Vec<WatchedFolder>> {
    let mut stmt = conn.prepare(SELECT_FOLDERS)?;
    let rows = stmt.query_map([], row_to_folder)?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(AppError::from)
}

fn folder_by_id(conn: &Connection, id: i64) -> AppResult<WatchedFolder> {
    let all = {
        let mut stmt = conn.prepare(SELECT_FOLDERS)?;
        let rows = stmt.query_map([], row_to_folder)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };
    all.into_iter()
        .find(|f| f.id == id)
        .ok_or_else(|| AppError::Invalid(format!("pasta {id} não encontrada")))
}

pub fn add_folder(conn: &Connection, path: &str) -> AppResult<WatchedFolder> {
    let canon = canonical(path)?;
    let label = label_for(&canon);
    conn.execute(
        "INSERT INTO watched_folders (path, label) VALUES (?1, ?2)
         ON CONFLICT(path) DO NOTHING",
        params![canon, label],
    )?;
    let id: i64 = conn.query_row(
        "SELECT id FROM watched_folders WHERE path = ?1",
        params![canon],
        |r| r.get(0),
    )?;
    folder_by_id(conn, id)
}

/// Usado pelo `Localizar…` da linha em erro: reaponta a pasta preservando o
/// registro (e portanto o histórico) em vez de forçar remover-e-readicionar.
pub fn update_folder_path(conn: &Connection, id: i64, path: &str) -> AppResult<WatchedFolder> {
    let canon = canonical(path)?;
    let label = label_for(&canon);
    let changed = conn.execute(
        "UPDATE watched_folders SET path = ?2, label = ?3 WHERE id = ?1",
        params![id, canon, label],
    )?;
    if changed == 0 {
        return Err(AppError::Invalid(format!("pasta {id} não encontrada")));
    }
    folder_by_id(conn, id)
}

pub fn remove_folder(conn: &Connection, id: i64) -> AppResult<()> {
    conn.execute("DELETE FROM watched_folders WHERE id = ?1", params![id])?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn list_watched_folders(db: State<'_, Db>) -> AppResult<Vec<WatchedFolder>> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    list_folders(&conn)
}

#[tauri::command]
#[specta::specta]
pub fn add_watched_folder(db: State<'_, Db>, path: String) -> AppResult<WatchedFolder> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    add_folder(&conn, &path)
}

#[tauri::command]
#[specta::specta]
pub fn update_watched_folder_path(
    db: State<'_, Db>,
    id: i64,
    path: String,
) -> AppResult<WatchedFolder> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    update_folder_path(&conn, id, &path)
}

#[tauri::command]
#[specta::specta]
pub fn remove_watched_folder(db: State<'_, Db>, id: i64) -> AppResult<()> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    remove_folder(&conn, id)
}

/// Cria uma pasta se ela não existir. Existe só pro preset "iCloud Drive ›
/// finan", que é a ÚNICA escrita em disco da feature — e mesmo assim só roda
/// depois da confirmação explícita do usuário na UI.
///
/// Três linhas aqui evitam adicionar o plugin `tauri-plugin-fs` inteiro
/// (que abriria acesso a arquivo muito além do necessário).
#[tauri::command]
#[specta::specta]
pub fn ensure_dir(path: String) -> AppResult<()> {
    std::fs::create_dir_all(&path)?;
    Ok(())
}

/// Permite à UI perguntar "criar a pasta?" apenas quando ela realmente não
/// existe, em vez de perguntar sempre.
#[tauri::command]
#[specta::specta]
pub fn dir_exists(path: String) -> bool {
    Path::new(&path).is_dir()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;

    fn fresh_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        migrations::apply(&conn).unwrap();
        conn
    }

    /// Pasta temporária única por teste, sem depender de crates externas.
    fn tmpdir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("finan-watch-test-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn add_folder_stores_canonical_path_and_label() {
        let conn = fresh_conn();
        let dir = tmpdir("add");
        let folder = add_folder(&conn, dir.to_str().unwrap()).unwrap();

        assert_eq!(folder.label, "finan-watch-test-add");
        assert!(folder.exists);
        assert_eq!(folder.imported_count, 0);
        assert_eq!(folder.last_imported_at, None);
        // canonicalize resolve /var → /private/var no macOS
        assert_eq!(folder.path, std::fs::canonicalize(&dir).unwrap().to_string_lossy());
    }

    #[test]
    fn add_folder_twice_is_idempotent() {
        let conn = fresh_conn();
        let dir = tmpdir("dup");
        let a = add_folder(&conn, dir.to_str().unwrap()).unwrap();
        let b = add_folder(&conn, dir.to_str().unwrap()).unwrap();

        assert_eq!(a.id, b.id, "readicionar devolve a mesma linha");
        assert_eq!(list_folders(&conn).unwrap().len(), 1);
    }

    #[test]
    fn add_folder_rejects_missing_path() {
        let conn = fresh_conn();
        let result = add_folder(&conn, "/caminho/que/nao/existe/mesmo");
        assert!(result.is_err());
    }

    #[test]
    fn add_folder_rejects_file() {
        let conn = fresh_conn();
        let dir = tmpdir("isfile");
        let file = dir.join("extrato.ofx");
        std::fs::write(&file, b"x").unwrap();
        let result = add_folder(&conn, file.to_str().unwrap());
        assert!(result.is_err(), "arquivo não pode ser adicionado como pasta");
    }

    #[test]
    fn missing_folder_reports_exists_false() {
        let conn = fresh_conn();
        let dir = tmpdir("gone");
        add_folder(&conn, dir.to_str().unwrap()).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();

        let folders = list_folders(&conn).unwrap();
        assert_eq!(folders.len(), 1);
        assert!(!folders[0].exists, "pasta removida deve reportar exists=false");
    }

    #[test]
    fn update_folder_path_preserves_row_id() {
        let conn = fresh_conn();
        let old = tmpdir("old");
        let new = tmpdir("new");
        let created = add_folder(&conn, old.to_str().unwrap()).unwrap();

        let moved = update_folder_path(&conn, created.id, new.to_str().unwrap()).unwrap();
        assert_eq!(moved.id, created.id, "Localizar… preserva o registro");
        assert_eq!(moved.label, "finan-watch-test-new");
        assert_eq!(list_folders(&conn).unwrap().len(), 1);
    }

    #[test]
    fn remove_folder_deletes_row() {
        let conn = fresh_conn();
        let dir = tmpdir("rm");
        let folder = add_folder(&conn, dir.to_str().unwrap()).unwrap();
        remove_folder(&conn, folder.id).unwrap();
        assert!(list_folders(&conn).unwrap().is_empty());
    }

    #[test]
    fn imported_count_reflects_seen_files_in_folder() {
        let conn = fresh_conn();
        let dir = tmpdir("count");
        let folder = add_folder(&conn, dir.to_str().unwrap()).unwrap();

        conn.execute(
            "INSERT INTO seen_files (content_hash, path, file_name, size, status, resolved_at)
             VALUES ('h1', ?1, 'a.ofx', 10, 'imported', '2026-07-02')",
            params![format!("{}/a.ofx", folder.path)],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO seen_files (content_hash, path, file_name, size, status, resolved_at)
             VALUES ('h2', ?1, 'b.ofx', 10, 'ignored', '2026-07-03')",
            params![format!("{}/b.ofx", folder.path)],
        )
        .unwrap();

        let folders = list_folders(&conn).unwrap();
        assert_eq!(folders[0].imported_count, 1, "só 'imported' conta");
        assert_eq!(folders[0].last_imported_at.as_deref(), Some("2026-07-02"));
    }

    #[test]
    fn ensure_dir_creates_and_is_idempotent() {
        let dir = tmpdir("ensure").join("finan");
        assert!(!dir.exists());

        ensure_dir(dir.to_string_lossy().to_string()).unwrap();
        assert!(dir.is_dir());

        // Rodar de novo numa pasta que já existe não pode falhar.
        ensure_dir(dir.to_string_lossy().to_string()).unwrap();
        assert!(dir.is_dir());
    }
}
